use std::fmt::Display;

use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

use crate::syntax::{
    database::{self, Summary},
    query::{Object, Predicate, Subject},
};

use super::operations::visitors::condition::ConditionInfo;

#[derive(Clone)]
pub enum SelectivityEstimator<'a> {
    // Do not change the given query
    Off,

    // Triples are evaluated in random order
    Random,

    // Triples are evaluated in random order
    Fixed,

    // Use the probabilistic framework selectivity estimation to order triples
    Arqpf(&'a Summary),

    // Similar to ARQ/PF, but also use variable distribution statistics
    Arqpfc(&'a Summary, &'a ConditionInfo),

    // Use the probabilistic framework selectivity estimation to order triples
    Arqpfj(&'a Summary),

    // Similar to ARQ/PFJ, but also use variable distribution statistics
    Arqpfjc(&'a Summary, &'a ConditionInfo),

    // Use Variable Counting to optimize queries
    Arqvc,

    // Use Variable Counting to optimize queries
    Arqvcp,
}

impl<'a> Display for SelectivityEstimator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectivityEstimator::Off => f.write_str("Off"),
            SelectivityEstimator::Random => f.write_str("Random"),
            SelectivityEstimator::Fixed => f.write_str("Fixed"),
            SelectivityEstimator::Arqpf(_) => f.write_str("ARQ/PF"),
            SelectivityEstimator::Arqpfc(_, _) => f.write_str("ARQ/PFC"),
            SelectivityEstimator::Arqpfj(_) => f.write_str("ARQ/PFJ"),
            SelectivityEstimator::Arqpfjc(_, _) => f.write_str("ARQ/PFJC"),
            SelectivityEstimator::Arqvc => f.write_str("ARQ/VC"),
            SelectivityEstimator::Arqvcp => f.write_str("ARQ/VCP"),
        }
    }
}

impl<'a> SelectivityEstimator<'a> {
    pub(crate) fn selectivity(&self, item: &(dyn Selectivity + 'a)) -> SelectivityResult {
        match self {
            SelectivityEstimator::Off => panic!("No selectivity for OFF Optimizer"),
            SelectivityEstimator::Random => item.sel_random(),
            SelectivityEstimator::Fixed => item.sel_fixed(),
            SelectivityEstimator::Arqpf(summary) => item.sel_pf(summary),
            SelectivityEstimator::Arqpfc(summary, infos) => item.sel_pfc(summary, infos),
            SelectivityEstimator::Arqpfj(summary) => item.sel_pfj(summary),
            SelectivityEstimator::Arqpfjc(summary, infos) => item.sel_pfjc(summary, infos),
            SelectivityEstimator::Arqvc => item.sel_vc(),
            SelectivityEstimator::Arqvcp => item.sel_vcp(),
        }
    }
}

#[derive(Debug)]
pub enum SelectivityError {
    NonConjunctiveStructure,
    EncounteredNaNValue,
    NoSelectivityForJoin,
}

pub type SelectivityResult = Result<f64, SelectivityError>;

pub(crate) trait Selectivity {
    fn sel_random(&self) -> SelectivityResult {
        Ok(thread_rng().sample(Uniform::new(0.0, 1.0)))
    }

    fn sel_fixed(&self) -> SelectivityResult {
        Ok(1.0)
    }

    fn sel_vc(&self) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_vc");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_vcp(&self) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_vcp");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pf(&self, _s: &database::Summary) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_pf");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pfc(&self, _s: &database::Summary, _i: &ConditionInfo) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_pfc");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pfj(&self, _s: &database::Summary) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_pfj");
        Err(SelectivityError::NonConjunctiveStructure)
    }

    fn sel_pfjc(&self, _s: &database::Summary, _i: &ConditionInfo) -> SelectivityResult {
        log::warn!("Hit default implementation for sel_pfjc");
        Err(SelectivityError::NonConjunctiveStructure)
    }
}

impl Selectivity for Subject {
    fn sel_vc(&self) -> SelectivityResult {
        let bound = match self {
            Subject::I(_) => true,
            Subject::V(_) => false,
        };

        let result = if bound { 0.25 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> SelectivityResult {
        let bound = match self {
            Subject::I(_) => true,
            Subject::V(_) => false,
        };

        let result = if bound { 1.0 / s.r() } else { 1.0 };

        Ok(result)
    }
}

impl Selectivity for Predicate {
    fn sel_vc(&self) -> SelectivityResult {
        let bound = match self {
            Predicate::I(_) => true,
            Predicate::V(_) => false,
        };

        let result = if bound { 0.75 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> SelectivityResult {
        let predicate = match self {
            Predicate::I(u) => Some(database::Predicate::I(u.to_owned())),
            Predicate::V(_) => None,
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
    fn sel_vc(&self) -> SelectivityResult {
        let bound = match self.1 {
            Object::L(_) | Object::I(_) => true,
            Object::V(_) => false,
        };

        let result = if bound { 0.5 } else { 1.0 };

        Ok(result)
    }

    fn sel_pf(&self, s: &Summary) -> SelectivityResult {
        let predicate = match self.0 {
            Predicate::I(u) => Some(database::Predicate::I(u.to_owned())),
            Predicate::V(_) => None,
        };

        let object = match self.1 {
            Object::L(l) => Some(database::Object::L(l.to_owned())),
            Object::I(u) => Some(database::Object::I(u.to_owned())),
            Object::V(_) => None,
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
