use std::fmt;

use crate::semantics::{mapping::Mapping, selectivity::Selectivity};

use super::{visitors::printer::Printer, Operation, OperationVisitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Union<O> {
    pub(crate) left: Box<O>,
    pub(crate) right: Box<O>,
}

impl<'a> Union<Operation<'a>> {
    pub(crate) fn new(left: Operation<'a>, right: Operation<'a>) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl<'a> fmt::Display for Union<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_union(self))
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Union<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Union next()");

        if let Some(next) = self.left.next() {
            log::trace!("Union next() returns left {next}");
            Some(next)
        } else if let Some(next) = self.right.next() {
            log::trace!("Union next() returns right {next}");
            Some(next)
        } else {
            log::trace!("Union next() returns None");
            None
        }
    }
}

impl<O> Selectivity for Union<O> {}
