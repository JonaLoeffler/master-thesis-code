use core::fmt;

use std::hash::Hash;

use crate::{
    semantics::{
        mapping::Mapping,
        selectivity::{Selectivity, SelectivityResult},
    },
    syntax::{database, query},
};

use super::{
    visitors::{condition, printer::Printer},
    OperationVisitor,
};

#[derive(Debug)]
pub(crate) struct Scan<'a> {
    pub(super) db: &'a database::Database,
    pub(super) subject: query::Subject,
    pub(super) predicate: query::Predicate,
    pub(super) object: query::Object,
    iter: std::iter::Cloned<std::slice::Iter<'a, database::Triple>>,
}

impl<'a> Scan<'a> {
    pub(crate) fn new(
        db: &'a database::Database,
        subject: query::Subject,
        predicate: query::Predicate,
        object: query::Object,
    ) -> Self {
        Self {
            db,
            subject,
            predicate,
            object,
            iter: db.triples().iter().cloned(),
        }
    }
}

impl<'a> Clone for Scan<'a> {
    fn clone(&self) -> Self {
        Self {
            db: self.db,
            subject: self.subject.clone(),
            predicate: self.predicate.clone(),
            object: self.object.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'a> Eq for Scan<'a> {}
impl<'a> PartialEq for Scan<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject
            && self.predicate == other.predicate
            && self.object == other.object
    }
}

impl<'a> Hash for Scan<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.subject.hash(state);
        self.predicate.hash(state);
        self.object.hash(state);
    }
}

impl<'a> fmt::Display for Scan<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_scan(self))
    }
}

impl Scan<'_> {
    fn triple_to_mapping(&self, triple: &database::Triple) -> Option<Mapping> {
        let subj = match &self.subject {
            query::Subject::I(s1) => match &triple.subject {
                database::Subject::B => false,
                database::Subject::I(s2) => s1 == s2,
            },
            query::Subject::V(_) => true,
        };

        let pred = match &self.predicate {
            query::Predicate::I(s1) => match &triple.predicate {
                database::Predicate::I(s2) => s1 == s2,
            },
            query::Predicate::V(_) => true,
        };

        let obj = match &self.object {
            query::Object::L(l1) => match &triple.object {
                database::Object::B => false,
                database::Object::L(l2) => l1 == l2,
                database::Object::I(_) => false,
            },
            query::Object::I(u1) => match &triple.object {
                database::Object::B => false,
                database::Object::L(_) => false,
                database::Object::I(u2) => u1 == u2,
            },
            query::Object::V(_) => true, // Match all instances where the query object is a variable
        };

        if subj && pred && obj {
            let mut result: Mapping = Mapping::new();

            if let query::Subject::V(v) = &self.subject {
                result.insert(
                    v.to_owned(),
                    match &triple.subject {
                        database::Subject::B => database::Object::B,
                        database::Subject::I(u) => database::Object::I(u.to_owned()),
                    },
                );
            }

            if let query::Predicate::V(v) = &self.predicate {
                result.insert(
                    v.to_owned(),
                    match &triple.predicate {
                        database::Predicate::I(i) => database::Object::I(i.to_owned()),
                    },
                );
            }

            if let query::Object::V(v) = &self.object {
                result.insert(v.to_owned(), triple.object.to_owned());
            }

            log::trace!("Scan next() returns {result}");

            Some(result)
        } else {
            None
        }
    }
}

impl<'a> Iterator for Scan<'a> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!(
            "Scan next() {} {} {}",
            self.subject,
            self.predicate,
            self.object
        );

        while let Some(triple) = self.iter.next() {
            if let Some(result) = self.triple_to_mapping(&triple) {
                return Some(result);
            };
        }

        log::trace!("Scan next() returns None");

        None
    }
}

impl<'a> Selectivity for Scan<'a> {
    fn sel_vc(&self) -> SelectivityResult {
        let sub = self.subject.sel_vc()?;
        let pre = self.predicate.sel_vc()?;
        let obj = (&self.predicate, &self.object).sel_vc()?;

        Ok(sub * pre * obj)
    }

    fn sel_vcp(&self) -> SelectivityResult {
        self.sel_vc()
    }

    fn sel_pf(&self, s: &database::Summary) -> SelectivityResult {
        let sub = self.subject.sel_pf(s)?;
        let pre = self.predicate.sel_pf(s)?;
        let obj = (&self.predicate, &self.object).sel_pf(s)?;

        Ok(sub * pre * obj)
    }

    fn sel_pfc(&self, s: &database::Summary, i: &condition::ConditionInfo) -> SelectivityResult {
        Ok(self.sel_pf(s)? * self.condition_factor(s, i))
    }

    fn sel_pfj(&self, s: &database::Summary) -> SelectivityResult {
        self.sel_pf(s)
    }

    fn sel_pfjc(&self, s: &database::Summary, i: &condition::ConditionInfo) -> SelectivityResult {
        Ok(self.sel_pfj(s)? * self.condition_factor(s, i))
    }
}

impl<'a> Scan<'a> {
    fn condition_factor(&self, s: &database::Summary, i: &condition::ConditionInfo) -> f64 {
        let mut lower = None;
        let mut upper = None;

        if let query::Object::V(v) = &self.object {
            if let Some(info) = i.get(v) {
                for info in info.iter() {
                    match info {
                        condition::VariableInfo::Lt(l) => {
                            upper = l.parsed.map(|x| x.min(upper.unwrap_or(f64::INFINITY)))
                        }
                        condition::VariableInfo::Gt(l) => {
                            lower = l.parsed.map(|x| x.max(lower.unwrap_or(-f64::INFINITY)))
                        }
                        condition::VariableInfo::Lte(l) => {
                            upper = l.parsed.map(|x| x.min(upper.unwrap_or(f64::INFINITY)))
                        }
                        condition::VariableInfo::Gte(l) => {
                            lower = l.parsed.map(|x| x.max(lower.unwrap_or(-f64::INFINITY)))
                        }
                        condition::VariableInfo::EqualsLiteral(l) => {
                            upper = l.parsed.map(|x| x.min(upper.unwrap_or(f64::INFINITY)));
                            lower = l.parsed.map(|x| x.min(lower.unwrap_or(-f64::INFINITY)));
                        }
                        _ => {}
                    }
                }
            }
        }

        let predicate = match &self.predicate {
            query::Predicate::I(i) => Some(database::Predicate::I(i.to_owned())),
            query::Predicate::V(_) => None,
        };

        match predicate {
            Some(p) => s.p_l(&p, lower, upper),
            None => 1.0,
        }
    }
}
