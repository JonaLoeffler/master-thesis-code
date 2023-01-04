use core::fmt;
use std::marker::PhantomData;

use crate::{
    semantics::{
        mapping::{Mapping, MappingSet},
        selectivity::{Selectivity, SelectivityError},
    },
    syntax::{database, query},
};

use super::{visitors::bound::BoundVars, Execute, OperationVisitor};

pub(crate) type NewScan<'a, S, J, M, L> = fn(
    &'a database::Database,
    query::Subject,
    query::Predicate,
    query::Object,
) -> Scan<'a, S, J, M, L>;

#[derive(Debug)]
pub(crate) struct Scan<'a, S, J, M, L> {
    pub(super) db: &'a database::Database,
    pub(super) subject: query::Subject,
    pub(super) predicate: query::Predicate,
    pub(super) object: query::Object,
    kind: S,
    phantom_join: PhantomData<J>,
    phantom_minus: PhantomData<M>,
    phantom_limit: PhantomData<L>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollScan;

#[derive(Debug, Clone)]
pub(crate) struct IterScan<'a> {
    iter: std::iter::Cloned<std::slice::Iter<'a, database::Triple>>,
}

impl<'a> Eq for IterScan<'a> {}
impl<'a> PartialEq for IterScan<'a> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<'a, J, M, L> Scan<'a, CollScan, J, M, L> {
    pub(crate) fn collection(
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
            kind: CollScan,
            phantom_join: PhantomData,
            phantom_minus: PhantomData,
            phantom_limit: PhantomData,
        }
    }
}

impl<'a, J, M, L> Scan<'a, IterScan<'a>, J, M, L> {
    pub(crate) fn iterator(
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
            kind: IterScan {
                iter: db.triples().iter().cloned(),
            },
            phantom_join: PhantomData,
            phantom_minus: PhantomData,
            phantom_limit: PhantomData,
        }
    }
}

impl<'a, S: Clone, J, M, L> Clone for Scan<'a, S, J, M, L> {
    fn clone(&self) -> Self {
        Self {
            db: self.db,
            subject: self.subject.clone(),
            predicate: self.predicate.clone(),
            object: self.object.clone(),
            kind: self.kind.clone(),
            phantom_join: PhantomData,
            phantom_minus: PhantomData,
            phantom_limit: PhantomData,
        }
    }
}

impl<'a, S, J, M, L> Eq for Scan<'a, S, J, M, L> {}
impl<'a, S, J, M, L> PartialEq for Scan<'a, S, J, M, L> {
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject
            && self.predicate == other.predicate
            && self.object == other.object
    }
}

impl<'a, S, J, M, L> fmt::Display for Scan<'a, S, J, M, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bound = BoundVars::new()
            .visit_scan(self)
            .iter()
            .cloned()
            .collect::<query::Variables>();

        f.write_str(&format!(
            "SCAN Bound: {}, BGP: {{ {} {} {} }}",
            bound, self.subject, self.predicate, self.object,
        ))
    }
}

impl<S, J, M, L> Scan<'_, S, J, M, L> {
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

impl<'a, J, M, L> Execute for Scan<'a, CollScan, J, M, L> {
    fn execute(&self) -> crate::semantics::mapping::MappingSet {
        let subject = &self.subject;
        let predicate = &self.predicate;
        let object = &self.object;
        let db = self.db;
        log::debug!("Scanning for {} {} {}", subject, predicate, object);

        let mappings: MappingSet = db
            .triples()
            .iter()
            .filter_map(|t| self.triple_to_mapping(t))
            .collect();

        log::debug!("Scan produces {} mappings", mappings.len());

        mappings
    }
}

impl<'a, J, M, L> Iterator for Scan<'a, IterScan<'a>, J, M, L> {
    type Item = Mapping;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.kind.iter.size_hint();
        let lower = if let Ok(selectivity) = self.sel_pf(self.db.summary()) {
            (lower as f32) * selectivity
        } else {
            0.0
        } as usize;

        (lower, upper)
    }

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!(
            "Scan next() {} {} {}",
            self.subject,
            self.predicate,
            self.object
        );

        while let Some(triple) = self.kind.iter.next() {
            if let Some(result) = self.triple_to_mapping(&triple) {
                return Some(result);
            };
        }

        log::trace!("Scan next() returns None");

        None
    }
}

impl<'a, S, J, M, L> Selectivity for Scan<'a, S, J, M, L> {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        let sub = self.subject.sel_vc()?;
        let pre = self.predicate.sel_vc()?;
        let obj = (&self.predicate, &self.object).sel_vc()?;

        Ok(sub * pre * obj)
    }

    fn sel_vcp(&self) -> Result<f32, SelectivityError> {
        self.sel_vc()
    }

    fn sel_pf(&self, s: &database::Summary) -> Result<f32, SelectivityError> {
        let sub = self.subject.sel_pf(s)?;
        let pre = self.predicate.sel_pf(s)?;
        let obj = (&self.predicate, &self.object).sel_pf(s)?;

        Ok(sub * pre * obj)
    }

    fn sel_pfj(&self, s: &database::Summary) -> Result<f32, SelectivityError> {
        self.sel_pf(s)
    }
}
