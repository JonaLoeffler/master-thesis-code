use std::{fmt::Display, time::Duration};

use super::{mapping::MappingSet, options::Optimizer};

#[derive(Debug, Default)]
pub struct QueryResult {
    kind: ResultType,
    run_duration: Option<Duration>,
    opt_duration: Option<Duration>,
    optimizers: Vec<Optimizer>,
    operations: Option<OperationMeta>,
}

#[derive(Debug, Default)]
pub struct OperationMeta {
    pub joins: usize,
    pub scans: usize,
    pub filters: usize,
    pub disjunct_joins: usize,
}

impl QueryResult {
    pub fn size(&self) -> usize {
        match &self.kind {
            ResultType::SelectResult(_, s) => s.to_owned(),
            ResultType::AskResult(_) => 1,
            ResultType::DryRun => 0,
        }
    }

    pub(crate) fn select(results: MappingSet) -> Self {
        let size = results.len();

        Self {
            kind: ResultType::SelectResult(results, size),
            run_duration: None,
            opt_duration: None,
            optimizers: vec![],
            operations: None,
        }
    }

    pub(crate) fn ask(result: bool) -> Self {
        Self {
            kind: ResultType::AskResult(result),
            ..Default::default()
        }
    }

    pub(crate) fn dryrun() -> Self {
        Self {
            kind: ResultType::DryRun,
            ..Default::default()
        }
    }

    pub(crate) fn with_run_duration(self, elapsed: Duration) -> QueryResult {
        Self {
            run_duration: Some(elapsed),
            ..self
        }
    }

    pub(crate) fn with_optimization_duration(self, elapsed: Duration) -> QueryResult {
        Self {
            opt_duration: Some(elapsed),
            ..self
        }
    }

    pub(crate) fn with_optimizers(self, optimizers: Vec<Optimizer>) -> QueryResult {
        Self { optimizers, ..self }
    }

    pub(crate) fn with_meta(self, meta: Option<OperationMeta>) -> QueryResult {
        Self {
            operations: meta,
            ..self
        }
    }

    pub(crate) fn discard_mappings(self) -> Self {
        Self {
            kind: match self.kind {
                ResultType::SelectResult(_, s) => ResultType::SelectResult(Vec::new(), s),
                ResultType::AskResult(r) => ResultType::AskResult(r),
                ResultType::DryRun => ResultType::DryRun,
            },
            ..self
        }
    }

    pub fn run_duration(&self) -> &Option<Duration> {
        &self.run_duration
    }

    pub fn opt_duration(&self) -> &Option<Duration> {
        &self.opt_duration
    }

    pub fn optimizers(&self) -> &Vec<Optimizer> {
        &self.optimizers
    }

    pub fn operations(&self) -> &Option<OperationMeta> {
        &self.operations
    }

    pub fn is_dryrun(&self) -> bool {
        match self.kind {
            ResultType::SelectResult(_, _) => false,
            ResultType::AskResult(_) => false,
            ResultType::DryRun => true,
        }
    }
}

impl PartialEq for QueryResult {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

#[derive(Debug, PartialEq, Default)]
enum ResultType {
    SelectResult(MappingSet, usize),
    AskResult(bool),
    #[default]
    DryRun,
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ResultType::SelectResult(results, _) => {
                for line in results {
                    for (_, val) in line.items.iter() {
                        f.write_str(&format!("{val} "))?;
                    }

                    f.write_str("\n")?;
                }
            }
            ResultType::AskResult(b) => f.write_str(&format!("{b}"))?,
            ResultType::DryRun => f.write_str("No results (Dry-Run)")?,
        };

        Ok(())
    }
}
