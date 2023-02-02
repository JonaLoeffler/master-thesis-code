use std::fmt;

use crate::semantics::{mapping::Mapping, selectivity::Selectivity};

use super::{visitors::printer::Printer, Operation, OperationVisitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Limit<O> {
    pub(crate) operation: Box<O>,
    pub(crate) limit: usize,
    current: usize,
}

impl<O> Limit<O> {
    pub(crate) fn new(operation: O, limit: usize) -> Self {
        Self {
            operation: Box::new(operation),
            limit,
            current: 0,
        }
    }
}

impl<'a> fmt::Display for Limit<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_limit(self))
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Limit<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.limit {
            self.current += 1;
            self.operation.next()
        } else {
            None
        }
    }
}

impl<O> Selectivity for Limit<O> {}
