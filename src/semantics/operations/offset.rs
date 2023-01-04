use std::fmt;

use crate::semantics::{
    mapping::{Mapping, MappingSet},
    selectivity::Selectivity,
};

use super::Execute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Offset<O> {
    pub(crate) operation: Box<O>,
    pub(crate) offset: usize,
}

impl<O> Offset<O> {
    pub(crate) fn new(operation: O, offset: usize) -> Self {
        Self {
            operation: Box::new(operation),
            offset,
        }
    }
}

impl<O: fmt::Display> fmt::Display for Offset<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("OFFSET {}", self.offset))?;

        f.write_str(&format!("\n{}", self.operation).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O: Execute> Execute for Offset<O> {
    fn execute(&self) -> MappingSet {
        self.operation
            .execute()
            .into_iter()
            .skip(self.offset)
            .collect()
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Offset<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset > 0 {
            self.operation.next();
            self.offset -= 1;
        }

        self.operation.next()
    }
}

impl<O> Selectivity for Offset<O> {}
