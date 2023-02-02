use std::fmt;

use crate::semantics::{mapping::Mapping, selectivity::Selectivity};

use super::{visitors::printer::Printer, Operation, OperationVisitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl<'a> fmt::Display for Offset<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_offset(self))
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
