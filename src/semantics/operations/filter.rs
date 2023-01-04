use std::fmt::{self, Display};

use crate::{
    semantics::{
        mapping::{Mapping, MappingSet},
        selectivity::Selectivity,
    },
    syntax::query,
};

use super::{Execute, Operation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Filter<O> {
    pub(crate) operation: Box<O>,
    pub(crate) condition: Box<query::Condition>,
}

impl<'a, S, J, M, L> Filter<Operation<'a, S, J, M, L>> {
    pub(crate) fn new(operation: Operation<'a, S, J, M, L>, condition: query::Condition) -> Self {
        Self {
            operation: Box::new(operation),
            condition: Box::new(condition),
        }
    }
}

impl<O: Display> Display for Filter<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("FILTER {}", self.condition))?;

        f.write_str(&format!("\n{}", self.operation).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O: Execute> Execute for Filter<O> {
    fn execute(&self) -> MappingSet {
        self.operation
            .execute()
            .into_iter()
            .filter(|m| m.satisfies(&self.condition))
            .collect()
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Filter<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        self.operation.find(|m| m.satisfies(&self.condition))
    }
}

impl<O> Selectivity for Filter<O> {}
