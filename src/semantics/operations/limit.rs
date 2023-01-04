use std::fmt;

use crate::semantics::{
    mapping::{Mapping, MappingSet},
    selectivity::Selectivity,
};

use super::{Execute, Operation};

pub(crate) type NewLimit<'a, S, J, M, L> =
    fn(Operation<'a, S, J, M, L>, limit: usize) -> Limit<L, Operation<'a, S, J, M, L>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Limit<L, O> {
    pub(crate) operation: Box<O>,
    pub(crate) limit: usize,
    kind: L,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IterLimit {
    current: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollLimit;

impl<O> Limit<IterLimit, O> {
    pub(crate) fn iterator(operation: O, limit: usize) -> Self {
        Self {
            operation: Box::new(operation),
            limit,
            kind: IterLimit { current: 0 },
        }
    }
}

impl<O> Limit<CollLimit, O> {
    pub(crate) fn collection(operation: O, limit: usize) -> Self {
        Self {
            operation: Box::new(operation),
            limit,
            kind: CollLimit,
        }
    }
}

impl<L, O: fmt::Display> fmt::Display for Limit<L, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("LIMIT {}", self.limit))?;

        f.write_str(&format!("\n{}", self.operation).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O: Execute> Execute for Limit<CollLimit, O> {
    fn execute(&self) -> MappingSet {
        self.operation
            .execute()
            .into_iter()
            .take(self.limit)
            .collect()
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Limit<IterLimit, O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        if self.kind.current < self.limit {
            self.kind.current += 1;
            self.operation.next()
        } else {
            None
        }
    }
}

impl<L, O> Selectivity for Limit<L, O> {}
