use std::fmt::Display;

use clap::ValueEnum;

#[derive(Clone)]
pub struct EvalOptions {
    pub optimizer: Optimizer,
    pub condition: bool,
    pub dryrun: bool,
    pub log: bool,
}

impl EvalOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_optimizer(self, optimizer: Optimizer) -> Self {
        Self { optimizer, ..self }
    }

    pub fn with_log(self, log: bool) -> Self {
        Self { log, ..self }
    }

    pub fn with_dryrun(self, dryrun: bool) -> Self {
        Self { dryrun, ..self }
    }

    pub fn with_condition(self, condition: bool) -> Self {
        Self { condition, ..self }
    }
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            optimizer: Optimizer::default(),
            condition: false,
            dryrun: false,
            log: true,
        }
    }
}

impl Display for EvalOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Evaluation Options:\n")?;
        f.write_str(&format!("Optimizer: {}\n", self.optimizer))?;
        f.write_str(&format!("Filter condition analysis: {}\n", self.condition))?;
        f.write_str(&format!("Dry-Run: {}\n", self.dryrun))?;
        f.write_str(&format!("Logging: {}\n", self.log))
    }
}

#[derive(Clone, Copy, ValueEnum, Hash, PartialEq, Eq, Debug, Default)]
pub enum Optimizer {
    // Do not change the given query
    Off,

    // Use random selectivity estimate
    Random,

    // Assign the same selectivity estimate to each element
    Fixed,

    // Use the probabilistic framework selectivity estimation to order triples
    Arqpf,

    // Similar to ARQ/PF, but also use variable distribution statistics
    Arqpfc,

    // Use the probabilistic framework selectivity estimation to order triples
    #[default]
    Arqpfj,

    // Similar to ARQ/PFJ, but also use variable distribution statistics
    Arqpfjc,

    // Use Variable Counting to optimize queries
    Arqvc,

    // Use Variable Counting to optimize queries
    Arqvcp,
}

impl Display for Optimizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Optimizer::Off => f.write_str("Off"),
            Optimizer::Random => f.write_str("Random"),
            Optimizer::Fixed => f.write_str("Fixed"),
            Optimizer::Arqpf => f.write_str("ARQ/PF"),
            Optimizer::Arqpfc => f.write_str("ARQ/PFC"),
            Optimizer::Arqpfj => f.write_str("ARQ/PFJ"),
            Optimizer::Arqpfjc => f.write_str("ARQ/PFJC"),
            Optimizer::Arqvc => f.write_str("ARQ/VC"),
            Optimizer::Arqvcp => f.write_str("ARQ/VCP"),
        }
    }
}
