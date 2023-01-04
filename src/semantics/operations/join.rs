use core::fmt;
use std::{collections::HashMap, fmt::Display, mem};

use crate::{
    semantics::{
        mapping::{Mapping, MappingSet},
        selectivity::{Selectivity, SelectivityError},
    },
    syntax::{database, query},
};
use iter_progress::ProgressableIter;

use super::{visitors::bound::BoundVars, Execute, Operation, OperationVisitor};

pub(crate) type NewJoin<'a, S, J, M, L> =
    fn(Operation<'a, S, J, M, L>, Operation<'a, S, J, M, L>) -> Join<J, Operation<'a, S, J, M, L>>;

#[derive(Debug, Clone)]
pub(crate) struct Join<J, O: Display> {
    pub(super) left: Box<O>,
    pub(super) right: Box<O>,
    join_vars: query::Variables,
    kind: J,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollJoin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IterJoin {
    hashes: HashMap<String, MappingSet>,
    current_bucket: MappingSet,
}

impl<'a, S, J, M, L> Join<J, Operation<'a, S, J, M, L>> {
    pub(crate) fn join_vars(&self) -> &query::Variables {
        &self.join_vars
    }
}

impl<'a, S, J, M, L> Join<IterJoin, Operation<'a, S, J, M, L>> {
    pub(crate) fn iterator(
        left: Operation<'a, S, J, M, L>,
        right: Operation<'a, S, J, M, L>,
    ) -> Self {
        let mut visitor = BoundVars::new();
        let left_vars = visitor.visit(&left);
        let right_vars = visitor.visit(&right);

        let join_vars = left_vars.intersection(&right_vars).cloned().collect();

        Self {
            left: Box::new(left),
            right: Box::new(right),
            join_vars,
            kind: IterJoin {
                hashes: HashMap::new(),
                current_bucket: Vec::new(),
            },
        }
    }
}

impl<'a, S, J, M, L> Join<CollJoin, Operation<'a, S, J, M, L>> {
    pub(crate) fn collection(
        left: Operation<'a, S, J, M, L>,
        right: Operation<'a, S, J, M, L>,
    ) -> Self {
        let mut visitor = BoundVars::new();
        let left_vars = visitor.visit(&left);
        let right_vars = visitor.visit(&right);

        let join_vars = left_vars.intersection(&right_vars).cloned().collect();

        Self {
            left: Box::new(left),
            right: Box::new(right),
            join_vars,
            kind: CollJoin,
        }
    }
}

impl<'a, J, O: Display + Eq> Eq for Join<J, O> {}
impl<'a, J, O: Display + PartialEq> PartialEq for Join<J, O> {
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left && self.right == other.right
    }
}

impl<'a, S, J, M, L> fmt::Display for Join<J, Operation<'a, S, J, M, L>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bound = BoundVars::new()
            .visit_join(self)
            .iter()
            .cloned()
            .collect::<query::Variables>();

        f.write_str(&format!("JOIN Bound: {}, Join: {}", bound, self.join_vars,))?;

        f.write_str(&format!("\n{}", self.left).replace("\n", "\n  "))?;
        f.write_str(&format!("\n{}", self.right).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<'a, O> Execute for Join<CollJoin, O>
where
    O: Execute,
    O: Display,
{
    fn execute(&self) -> MappingSet {
        let left = self.left.execute();
        let right = self.right.execute();

        let o1 = left;
        let o2 = right;
        log::debug!("Joining {} {} ", o1.len(), o2.len());

        log::debug!("Joining on common keys {}", self.join_vars);
        log::debug!("Hash join build phase");

        let mut hashes: HashMap<String, MappingSet> = HashMap::new();

        let mut o1 = o1;
        let mut o2 = o2;

        if o1.len() > o2.len() {
            mem::swap(&mut o1, &mut o2);
        }

        log::debug!("Building hash join table from {} entries", o1.len());

        o1.into_iter().progress().for_each(|(state, m)| {
            state.do_every_n_sec(5., |s| {
                log::debug!(
                    "{:.2}% done with building hash join table, {:.2} per sec.",
                    s.percent().unwrap(),
                    s.rate()
                );
            });

            let key = m.hash_map_key(&self.join_vars);

            if let Some(v) = hashes.get_mut(&key) {
                v.push(m);
            } else {
                hashes.insert(key, vec![m]);
            };
        });

        log::debug!("Hash join table entries: {}", hashes.len());

        log::debug!("Hash join probe phase");

        let mappings: MappingSet = o2
            .into_iter()
            .progress()
            .flat_map(|(state, m)| {
                state.do_every_n_sec(5., |s| {
                    log::debug!(
                        "{:.2}% done with hash join probe, {:.2} per sec.",
                        s.percent().unwrap(),
                        s.rate()
                    );
                });

                let key = m.hash_map_key(&self.join_vars);

                let or = Vec::new();

                hashes
                    .get(&key)
                    .unwrap_or(&or)
                    .into_iter()
                    .map(move |other| {
                        let mut next = Mapping::new();
                        for (k, v) in m.items.iter() {
                            next.insert(k.clone(), v.clone());
                        }
                        for (k, v) in other.items.iter() {
                            next.insert(k.clone(), v.clone());
                        }
                        next
                    })
                    .collect::<MappingSet>()
            })
            .collect();

        log::debug!("Join produces {} mappings", mappings.len());

        mappings
    }
}

impl<'a, O> Iterator for Join<IterJoin, O>
where
    O: Iterator<Item = Mapping>,
    O: Display,
{
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Join next()");

        if self.kind.hashes.is_empty() {
            log::debug!("Building hash table");

            while let Some(m) = self.left.next() {
                let key = m.hash_map_key(&self.join_vars);

                if let Some(v) = self.kind.hashes.get_mut(&key) {
                    v.push(m);
                } else {
                    self.kind.hashes.insert(key, vec![m]);
                };
            }

            log::debug!("Hash table has {} entries", self.kind.hashes.len());
        }

        while self.kind.current_bucket.is_empty() {
            log::debug!("Building next bucket");

            if let Some(m) = self.right.next() {
                let key = m.hash_map_key(&self.join_vars);
                log::trace!("Join key {:?}", key);

                let or = Vec::new();

                self.kind.current_bucket = self
                    .kind
                    .hashes
                    .get(&key)
                    .unwrap_or(&or)
                    .into_iter()
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

            log::debug!("Bucket now has {} entries", self.kind.current_bucket.len());
        }

        if let Some(result) = self.kind.current_bucket.pop() {
            log::trace!("Join next() returns {result}");
            Some(result)
        } else {
            log::trace!("Join next() returns None");
            None
        }
    }
}

impl<'a, S, J, M, L> Selectivity for Join<J, Operation<'a, S, J, M, L>> {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        Ok(self.left.sel_vc()? * self.right.sel_vc()?)
    }

    fn sel_vcp(&self) -> Result<f32, SelectivityError> {
        if let (Operation::Scan(l), Operation::Scan(r)) = (self.left.as_ref(), self.right.as_ref())
        {
            if let (query::Subject::V(v1), query::Subject::V(v2)) = (&l.subject, &r.subject) {
                if v1 != v2 {
                    return Ok(1.0);
                }
            }
        }

        self.sel_vc()
    }

    fn sel_pf(&self, s: &database::Summary) -> Result<f32, SelectivityError> {
        if self.join_vars.is_empty() {
            return Err(SelectivityError::NoSelectivityForJoin);
        }

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

        let leftsize = leftscan.db.triples().len() as f32;
        let rightsize = rightscan.db.triples().len() as f32;

        let s_p = leftsize * rightsize;

        let factor = match leftscan.object {
            query::Object::L(_) | query::Object::I(_) => (&leftscan.predicate, &leftscan.object)
                .sel_pf(s)
                .unwrap_or(1.0),
            query::Object::V(_) => 1.0,
        };

        Ok(s_p / (s.t() * s.t()) * factor)
    }

    fn sel_pfj(&self, s: &database::Summary) -> Result<f32, SelectivityError> {
        if let (Operation::Scan(l), Operation::Scan(r)) = (&*self.left, &*self.right) {
            if let (query::Subject::V(v1), query::Subject::V(v2)) = (&l.subject, &r.subject) {
                if v1 != v2 {
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
}
