use std::fmt::Display;

use clap::ValueEnum;

#[derive(Clone)]
pub struct EvalOptions {
    pub semantics: Semantics,
    pub optimizer: Optimizer,
    pub dryrun: bool,
}

impl EvalOptions {
    pub fn new(semantics: Semantics, optimizer: Optimizer, dryrun: bool) -> Self {
        Self {
            optimizer,
            semantics,
            dryrun,
        }
    }

    pub fn with_semantics(self, semantics: Semantics) -> Self {
        Self {
            semantics,
            optimizer: self.optimizer,
            dryrun: self.dryrun,
        }
    }

    pub fn with_optimizer(self, optimizer: Optimizer) -> Self {
        Self {
            semantics: self.semantics,
            optimizer,
            dryrun: self.dryrun,
        }
    }
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            semantics: Semantics::Iterator,
            optimizer: Optimizer::ARQPF,
            dryrun: false,
        }
    }
}

impl Display for EvalOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Evaluation Options:\n")?;
        f.write_str(&format!("Semantics: {}\n", self.semantics))?;
        f.write_str(&format!("Optimizer: {}\n", self.optimizer))?;
        f.write_str(&format!("Dry-Run: {}\n", self.dryrun))
    }
}

#[derive(Clone, Copy, ValueEnum, Hash, PartialEq, Eq, Debug)]
pub enum Optimizer {
    // Do not change the given query
    Off,

    // Use random selectivity estimate
    Random,

    // Assign the same selectivity estimate to each element
    Fixed,

    // Use the probabilistic framework selectivity estimation to order triples
    ARQPF,

    // Use the probabilistic framework selectivity estimation to order triples
    ARQPFJ,

    // Use Variable Counting to optimize queries
    ARQVC,

    // Use Variable Counting to optimize queries
    ARQVCP,
}

impl Display for Optimizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Optimizer::Off => f.write_str("Off"),
            Optimizer::Random => f.write_str("Random"),
            Optimizer::Fixed => f.write_str("Fixed"),
            Optimizer::ARQPF => f.write_str("ARQ/PF"),
            Optimizer::ARQPFJ => f.write_str("ARQ/PFJ"),
            Optimizer::ARQVC => f.write_str("ARQ/VC"),
            Optimizer::ARQVCP => f.write_str("ARQ/VCP"),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Semantics {
    Iterator,
    Collection,
}

impl Display for Semantics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Semantics::Iterator => f.write_str("Iterator"),
            Semantics::Collection => f.write_str("Collection"),
        }
    }
}
