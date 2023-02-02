use core::fmt;
use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::{
    semantics::{
        mapping::{Mapping, MappingSet},
        selectivity::{Selectivity, SelectivityError, SelectivityResult},
    },
    syntax::{
        database,
        query::{self, Object, Predicate, Subject},
    },
};
use iter_progress::ProgressableIter;

use super::{
    visitors::{condition::ConditionInfo, printer::Printer},
    Operation, OperationVisitor,
};

#[derive(Debug, Clone)]
pub(crate) struct Join<O: Display> {
    pub(super) left: Box<O>,
    pub(super) right: Box<O>,
    pub(super) join_vars: query::Variables,
    hashes: HashMap<String, MappingSet>,
    current_bucket: MappingSet,
}

impl<O: Hash + Display> Hash for Join<O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.right.hash(state);
    }
}

impl<'a> Join<Operation<'a>> {
    pub(crate) fn join_vars(&self) -> &query::Variables {
        &self.join_vars
    }
}

impl<'a> Join<Operation<'a>> {
    pub(crate) fn new(left: Operation<'a>, right: Operation<'a>) -> Self {
        let join_vars = left
            .bound_vars()
            .intersection(&right.bound_vars())
            .cloned()
            .collect();

        Self {
            left: Box::new(left),
            right: Box::new(right),
            join_vars,
            hashes: HashMap::new(),
            current_bucket: Vec::new(),
        }
    }
}

impl<O: Display + Eq> Eq for Join<O> {}
impl<O: Display + PartialEq> PartialEq for Join<O> {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl<'a> fmt::Display for Join<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_join(self))
    }
}

impl<O> Iterator for Join<O>
where
    O: Iterator<Item = Mapping>,
    O: Display,
{
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Join next()");

        if self.hashes.is_empty() {
            log::debug!("Building hash table");

            for m in self.left.by_ref() {
                let key = m.hash_map_key(&self.join_vars);

                if let Some(v) = self.hashes.get_mut(&key) {
                    v.push(m);
                } else {
                    self.hashes.insert(key, vec![m]);
                };
            }

            log::debug!("Hash table has {} entries", self.hashes.len());
        }

        while self.current_bucket.is_empty() {
            log::debug!("Building next bucket");

            if let Some(m) = self.right.next() {
                let key = m.hash_map_key(&self.join_vars);
                log::trace!("Join key {:?}", key);

                let or = Vec::new();

                self.current_bucket = self
                    .hashes
                    .get(&key)
                    .unwrap_or(&or)
                    .iter()
                    .progress()
                    .map(move |(state, other)| {
                        state.do_every_n_sec(5., |s| {
                            log::debug!(
                                "{:.2}% done with next bucket, {:.2} per sec.",
                                s.percent().unwrap(),
                                s.rate()
                            );
                        });

                        let mut next = Mapping::new();
                        for (k, v) in m.items.iter() {
                            next.insert(k.clone(), v.clone());
                        }
                        for (k, v) in other.items.iter() {
                            next.insert(k.clone(), v.clone());
                        }
                        next
                    })
                    .collect::<MappingSet>();
            } else {
                break;
            }

            log::debug!("Bucket now has {} entries", self.current_bucket.len());
        }

        if let Some(result) = self.current_bucket.pop() {
            log::trace!("Join next() returns {result}");
            Some(result)
        } else {
            log::trace!("Join next() returns None");
            None
        }
    }
}

pub(crate) enum JoinType {
    SubjectSubject,
    SubjectObject,
    SubjectPredicate,
    ObjectObject,
    UnboundOrHigherJoin,
}

impl<'a> Join<Operation<'a>> {
    pub(crate) fn join_type(&self) -> Vec<JoinType> {
        if let (Operation::Scan(left), Operation::Scan(right)) = (&*self.left, &*self.right) {
            let mut types = Vec::new();

            if let (Object::V(l), Object::V(r)) = (&left.object, &right.object) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::ObjectObject);
                }
            }
            if let (Subject::V(l), Subject::V(r)) = (&left.subject, &right.subject) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectSubject);
                }
            }

            if let (Subject::V(l), Object::V(r)) = (&left.subject, &right.object) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectObject);
                }
            }
            if let (Object::V(l), Subject::V(r)) = (&left.object, &right.subject) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectObject);
                }
            }

            if let (Subject::V(l), Subject::V(r)) = (&left.subject, &right.subject) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectSubject);
                }
            }

            if let (Subject::V(l), Predicate::V(r)) = (&left.subject, &right.predicate) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectPredicate);
                }
            }
            if let (Predicate::V(l), Subject::V(r)) = (&left.predicate, &right.subject) {
                if l == r && self.join_vars.iter().any(|j| j == l) {
                    types.push(JoinType::SubjectPredicate);
                }
            }

            if !types.is_empty() {
                return types;
            }
        }

        vec![JoinType::UnboundOrHigherJoin]
    }
}

impl<'a> Selectivity for Join<Operation<'a>> {
    fn sel_vc(&self) -> SelectivityResult {
        let factor: f64 = self
            .join_type()
            .iter()
            .map(|t| match t {
                JoinType::SubjectPredicate => 0.25,

                JoinType::SubjectSubject => 0.5,

                JoinType::ObjectObject => 0.75,
                JoinType::SubjectObject => 0.75,

                JoinType::UnboundOrHigherJoin => 1.0,
            })
            .fold(1.0, |a, b| a.min(b));

        Ok(factor * self.left.sel_vc()? * self.right.sel_vc()?)
    }

    fn sel_vcp(&self) -> SelectivityResult {
        if let (Operation::Scan(l), Operation::Scan(r)) = (self.left.as_ref(), self.right.as_ref())
        {
            if let (Subject::V(_), Subject::V(_)) = (&l.subject, &r.subject) {
                if match l.object {
                    Object::L(_) | Object::I(_) => matches!(r.object, Object::L(_) | Object::I(_)),
                    _ => false,
                } {
                    return Ok(1.0);
                }
            }
        }

        self.sel_vc()
    }

    fn sel_pf(&self, s: &database::Summary) -> SelectivityResult {
        let leftscan = if let Operation::Scan(leftscan) = &*self.left {
            leftscan
        } else {
            return Err(SelectivityError::NoSelectivityForJoin);
        };

        let rightscan = if let Operation::Scan(rightscan) = &*self.right {
            rightscan
        } else {
            return Err(SelectivityError::NoSelectivityForJoin);
        };

        let p1 = if let Predicate::I(i) = &leftscan.predicate {
            database::Predicate::I(i.to_owned())
        } else {
            return Ok(1.0);
        };

        let p2 = if let Predicate::I(i) = &rightscan.predicate {
            database::Predicate::I(i.to_owned())
        } else {
            return Ok(1.0);
        };

        let mut factors = Vec::new();
        factors.push(match leftscan.object {
            query::Object::L(_) | query::Object::I(_) => {
                (&leftscan.predicate, &leftscan.object).sel_pf(s)?
            }
            query::Object::V(_) => 1.0,
        });
        factors.push(match rightscan.object {
            query::Object::L(_) | query::Object::I(_) => {
                (&rightscan.predicate, &rightscan.object).sel_pf(s)?
            }
            query::Object::V(_) => 1.0,
        });
        factors.push(match leftscan.subject {
            query::Subject::I(_) => leftscan.sel_pf(s)?,
            query::Subject::V(_) => 1.0,
        });
        factors.push(match rightscan.subject {
            query::Subject::I(_) => rightscan.sel_pf(s)?,
            query::Subject::V(_) => 1.0,
        });

        Ok(s.s_p(p1, p2) / (s.t() * s.t()) * factors.iter().product::<f64>())
    }

    fn sel_pfc(&self, s: &database::Summary, i: &ConditionInfo) -> SelectivityResult {
        let leftscan = if let Operation::Scan(leftscan) = &*self.left {
            leftscan
        } else {
            return Err(SelectivityError::NoSelectivityForJoin);
        };

        let rightscan = if let Operation::Scan(rightscan) = &*self.right {
            rightscan
        } else {
            return Err(SelectivityError::NoSelectivityForJoin);
        };

        let p1 = if let Predicate::I(i) = &leftscan.predicate {
            database::Predicate::I(i.to_owned())
        } else {
            return Ok(1.0);
        };

        let p2 = if let Predicate::I(i) = &rightscan.predicate {
            database::Predicate::I(i.to_owned())
        } else {
            return Ok(1.0);
        };

        let mut factors = Vec::new();
        factors.push(match leftscan.object {
            query::Object::L(_) | query::Object::I(_) => {
                (&leftscan.predicate, &leftscan.object).sel_pf(s)?
            }
            query::Object::V(_) => 1.0,
        });
        factors.push(match rightscan.object {
            query::Object::L(_) | query::Object::I(_) => {
                (&rightscan.predicate, &rightscan.object).sel_pf(s)?
            }
            query::Object::V(_) => 1.0,
        });
        factors.push(leftscan.sel_pfc(s, i)?);
        factors.push(rightscan.sel_pfc(s, i)?);

        Ok(s.s_p(p1, p2) / (s.t() * s.t()) * factors.iter().product::<f64>())
    }

    fn sel_pfj(&self, s: &database::Summary) -> SelectivityResult {
        if let (Operation::Scan(l), Operation::Scan(r)) = (&*self.left, &*self.right) {
            if let (Subject::V(_), Subject::V(_)) = (&l.subject, &r.subject) {
                let early_return = match l.object {
                    Object::L(_) | Object::I(_) => matches!(r.object, Object::L(_) | Object::I(_)),
                    _ => false,
                };

                if early_return {
                    return Ok(1.0);
                }
            }

            let join = self.sel_pf(s)?;
            let left = l.sel_pfj(s)?;
            let right = r.sel_pfj(s)?;

            let more_selective = left.min(right);

            return Ok(join * more_selective);
        }

        Err(SelectivityError::NoSelectivityForJoin)
    }

    fn sel_pfjc(&self, s: &database::Summary, i: &ConditionInfo) -> SelectivityResult {
        if let (Operation::Scan(l), Operation::Scan(r)) = (&*self.left, &*self.right) {
            if let (Subject::V(_), Subject::V(_)) = (&l.subject, &r.subject) {
                let early_return = match l.object {
                    Object::L(_) | Object::I(_) => matches!(r.object, Object::L(_) | Object::I(_)),
                    _ => false,
                };

                if early_return {
                    return Ok(1.0);
                }
            }

            let join = self.sel_pfc(s, i)?;
            let left = l.sel_pfjc(s, i)?;
            let right = r.sel_pfjc(s, i)?;

            let more_selective = left.min(right);

            return Ok(join * more_selective);
        }

        Err(SelectivityError::NoSelectivityForJoin)
    }
}
