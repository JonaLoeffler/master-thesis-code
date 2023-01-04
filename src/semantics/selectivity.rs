use std::fmt::Display;

use log::debug;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

use crate::syntax::{
    database::{self, Summary},
    query::{Expression, Object, Predicate, Query, Subject, Type},
};

#[derive(Clone)]
pub enum SelectivityEstimator<'a> {
    // Do not change the given query
    Off,

    // Triples are evaluated in random order
    Random,

    // Triples are evaluated in random order
    Fixed,

    // Use the probabilistic framework selectivity estimation to order triples
    ARQPF(&'a Summary),

    // Use the probabilistic framework selectivity estimation to order triples
    ARQPFJ(&'a Summary),

    // Use Variable Counting to optimize queries
    ARQVC,

    // Use Variable Counting to optimize queries
    ARQVCP,
}

impl<'a> Display for SelectivityEstimator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectivityEstimator::Off => f.write_str("Off"),
            SelectivityEstimator::Random => f.write_str("Random"),
            SelectivityEstimator::Fixed => f.write_str("Fixed"),
            SelectivityEstimator::ARQPF(_) => f.write_str("ARQ/PF"),
            SelectivityEstimator::ARQPFJ(_) => f.write_str("ARQ/PFJ"),
            SelectivityEstimator::ARQVC => f.write_str("ARQ/VC"),
            SelectivityEstimator::ARQVCP => f.write_str("ARQ/VCP"),
        }
    }
}

impl<'a> SelectivityEstimator<'a> {
    pub(crate) fn selectivity(
        &self,
        item: Box<&(dyn Selectivity + 'a)>,
    ) -> Result<f32, SelectivityError> {
        match self {
            SelectivityEstimator::Off => panic!("No selectivity for OFF Optimizer"),
            SelectivityEstimator::Random => item.sel_random(),
            SelectivityEstimator::Fixed => item.sel_fixed(),
            SelectivityEstimator::ARQPF(summary) => item.sel_pf(summary),
            SelectivityEstimator::ARQPFJ(summary) => item.sel_pfj(summary),
            SelectivityEstimator::ARQVC => item.sel_vc(),
            SelectivityEstimator::ARQVCP => item.sel_vcp(),
        }
    }
}

pub(crate) trait Selectivity {
    fn sel_random(&self) -> Result<f32, SelectivityError> {
        Ok(thread_rng().sample(Uniform::new(0.0, 1.0)))
    }

    fn sel_fixed(&self) -> Result<f32, SelectivityError> {
        Ok(1.0)
    }

    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        log::warn!("Hit default implementation for sel_vc");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_vcp(&self) -> Result<f32, SelectivityError> {
        log::warn!("Hit default implementation for sel_vcp");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pf(&self, _s: &database::Summary) -> Result<f32, SelectivityError> {
        log::warn!("Hit default implementation for sel_pf");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pfj(&self, _s: &database::Summary) -> Result<f32, SelectivityError> {
        log::warn!("Hit default implementation for sel_pfj");
        Err(SelectivityError::NonConjunctiveStructure)
    }
}

#[derive(Debug)]
pub enum SelectivityError {
    NonConjunctiveStructure,
    EncounteredNaNValue,
    NoSelectivityForJoin,
}

impl Selectivity for Subject {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        let bound = match self {
            Subject::I(_) => true,
            Subject::V(_) => false, // TODO: ?
        };

        let result = if bound { 0.25 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> Result<f32, SelectivityError> {
        let bound = match self {
            Subject::I(_) => true,
            Subject::V(_) => false, // TODO: ?
        };

        let result = if bound { 1.0 / s.r() } else { 1.0 };

        Ok(result)
    }
}

impl Selectivity for Predicate {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        let bound = match self {
            Predicate::I(_) => true,
            Predicate::V(_) => false, // TODO: ?
        };

        let result = if bound { 0.75 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> Result<f32, SelectivityError> {
        let predicate = match self {
            Predicate::I(u) => Some(database::Predicate::I(u.to_owned())),
            Predicate::V(_) => None, // TODO: ?
        };

        let result = if let Some(p) = predicate {
            s.t_p(&p) / s.t()
        } else {
            1.0
        };

        if !result.is_nan() {
            Ok(result)
        } else {
            Err(SelectivityError::EncounteredNaNValue)
        }
    }
}

impl Selectivity for (&Predicate, &Object) {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        let bound = match self.1 {
            Object::L(_) => true,
            Object::I(_) => true,
            Object::V(_) => false, // TODO: ?
        };

        let result = if bound { 0.5 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> Result<f32, SelectivityError> {
        let predicate = match self.0 {
            Predicate::I(u) => Some(database::Predicate::I(u.to_owned())),
            Predicate::V(_) => None, // TODO: ?
        };

        let object = match self.1 {
            Object::L(l) => Some(database::Object::L(l.to_owned())),
            Object::I(u) => Some(database::Object::I(u.to_owned())),
            Object::V(_) => None, // TODO: ?
        };

        let result = if let Some(o) = object {
            if let Some(p) = predicate {
                s.o_c(&p, &o) / s.t_p(&p)
            } else {
                s.o_c.keys().map(|p| s.o_c(p, &o) / s.t_p(p)).sum()
            }
        } else {
            1.0
        };

        if !result.is_nan() {
            Ok(result)
        } else {
            Err(SelectivityError::EncounteredNaNValue)
        }
    }
}

impl Selectivity for Query {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        match &self.kind {
            Type::SelectQuery(_, e, _) => e.sel_vc(),
            Type::AskQuery(e, _) => e.sel_vc(),
        }
    }

    fn sel_pf(&self, s: &Summary) -> Result<f32, SelectivityError> {
        match &self.kind {
            Type::SelectQuery(_, e, _) => e.sel_pf(s),
            Type::AskQuery(e, _) => e.sel_pf(s),
        }
    }
}

impl Selectivity for Expression {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        match self {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => {
                let sub = subject.sel_vc()?;
                let pre = predicate.sel_vc()?;
                let obj = (predicate, object).sel_vc()?;

                debug!(
                    "sel(s) = {:.5}, sel(p) = {:.5}, sel(o) = {:.5}, sel(t) = {:.5}",
                    sub,
                    pre,
                    obj,
                    sub * pre * obj
                );

                Ok(sub * pre * obj)
            }
            Expression::And(e1, e2) => Ok(e1.sel_vc()? * e2.sel_vc()?),
            Expression::Union(_, _) => Err(SelectivityError::NonConjunctiveStructure),
            Expression::Optional(_, _) => Err(SelectivityError::NonConjunctiveStructure),
            Expression::Filter(_, _) => Err(SelectivityError::NonConjunctiveStructure),
        }
    }

    fn sel_pf(&self, s: &Summary) -> Result<f32, SelectivityError> {
        match self {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => {
                let sub = subject.sel_pf(s)?;
                let pre = predicate.sel_pf(s)?;
                let obj = (predicate, object).sel_pf(s)?;

                debug!(
                    "sel(s) = {:.5}, sel(p) = {:.5}, sel(o) = {:.5}, sel(t) = {:.5}",
                    sub,
                    pre,
                    obj,
                    sub * pre * obj
                );

                Ok(sub * pre * obj)
            }
            Expression::And(e1, e2) => Ok(e1.sel_pf(s)? * e2.sel_pf(s)?),
            _ => Err(SelectivityError::NonConjunctiveStructure),
        }
    }
}
