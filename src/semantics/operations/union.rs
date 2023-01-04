use std::fmt::{self, Display};

use crate::semantics::{mapping, mapping::Mapping, selectivity::Selectivity};

use super::{Execute, Operation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Union<O> {
    pub(crate) left: Box<O>,
    pub(crate) right: Box<O>,
}

impl<'a, S, J, M, L> Union<Operation<'a, S, J, M, L>> {
    pub(crate) fn new(left: Operation<'a, S, J, M, L>, right: Operation<'a, S, J, M, L>) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl<O: Display> fmt::Display for Union<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("UNION"))?;

        f.write_str(&format!("\n{}", self.left).replace("\n", "\n  "))?;
        f.write_str(&format!("\n{}", self.right).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O: Execute> Execute for Union<O> {
    fn execute(&self) -> mapping::MappingSet {
        let mut left = self.left.execute();
        let mut right = self.right.execute();

        left.append(&mut right);

        left
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Union<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Union next()");

        if let Some(next) = self.left.next() {
            log::trace!("Union next() returns left {next}");
            Some(next)
        } else {
            if let Some(next) = self.right.next() {
                log::trace!("Union next() returns right {next}");
                Some(next)
            } else {
                log::trace!("Union next() returns None");
                None
            }
        }
    }
}

impl<O> Selectivity for Union<O> {}
