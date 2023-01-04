use std::{fmt::Display, time::Duration};

use super::{mapping::MappingSet, options::Optimizer};

#[derive(Debug)]
pub struct QueryResult {
    kind: ResultType,
    duration: Option<Duration>,
    optimizers: Vec<Optimizer>,
    operations: Option<OperationMeta>,
}

#[derive(Debug)]
pub struct OperationMeta {
    pub joins: usize,
    pub scans: usize,
    pub disjunct_joins: usize,
}

impl QueryResult {
    pub fn size(&self) -> usize {
        match &self.kind {
            ResultType::SelectResult(s) => s.len(),
            ResultType::AskResult(_) => 1,
            ResultType::DryRun => 0,
        }
    }

    pub(crate) fn select(results: MappingSet) -> Self {
        Self {
            kind: ResultType::SelectResult(results),
            duration: None,
            optimizers: vec![],
            operations: None,
        }
    }

    pub(crate) fn ask(result: bool) -> Self {
        Self {
            kind: ResultType::AskResult(result),
            duration: None,
            optimizers: vec![],
            operations: None,
        }
    }

    pub(crate) fn dryrun() -> Self {
        Self {
            kind: ResultType::DryRun,
            duration: None,
            optimizers: vec![],
            operations: None,
        }
    }

    pub(crate) fn with_duration(self, elapsed: Duration) -> QueryResult {
        Self {
            kind: self.kind,
            duration: Some(elapsed),
            optimizers: self.optimizers,
            operations: self.operations,
        }
    }

    pub(crate) fn with_optimizers(self, optimizers: Vec<Optimizer>) -> QueryResult {
        Self {
            kind: self.kind,
            duration: self.duration,
            optimizers,
            operations: self.operations,
        }
    }

    pub(crate) fn with_meta(self, meta: Option<OperationMeta>) -> QueryResult {
        Self {
            kind: self.kind,
            duration: self.duration,
            optimizers: self.optimizers,
            operations: meta,
        }
    }

    pub fn duration(&self) -> &Option<Duration> {
        &self.duration
    }

    pub fn optimizers(&self) -> &Vec<Optimizer> {
        &self.optimizers
    }

    pub fn operations(&self) -> &Option<OperationMeta> {
        &self.operations
    }
}

impl PartialEq for QueryResult {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

#[derive(Debug, PartialEq)]
enum ResultType {
    SelectResult(MappingSet),
    AskResult(bool),
    DryRun,
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ResultType::SelectResult(results) => {
                for line in results {
                    for (_, val) in line.items.iter() {
                        f.write_str(&format!("{} ", val))?;
                    }

                    f.write_str("\n")?;
                }
            }
            ResultType::AskResult(b) => f.write_str(&format!("{}", b))?,
            ResultType::DryRun => f.write_str("No results (Dry-Run)")?,
        };

        Ok(())
    }
}
