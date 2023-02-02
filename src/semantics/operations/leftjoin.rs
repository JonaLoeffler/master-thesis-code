use std::fmt::{self, Display};

use crate::semantics::{mapping::Mapping, selectivity::Selectivity};

use super::{
    join::Join, minus::Minus, union::Union, visitors::printer::Printer, Operation, OperationVisitor,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LeftJoin<O> {
    pub(crate) operation: Box<O>,
    pub(crate) left: Box<O>,
    pub(crate) right: Box<O>,
}

impl<'a> LeftJoin<Operation<'a>> {
    pub(crate) fn new(left: Operation<'a>, right: Operation<'a>) -> Self {
        let operation = Box::new(Operation::Union(Union::new(
            Operation::Join(Join::new(left.clone(), right.clone())),
            Operation::Minus(Minus::new(left.clone(), right.clone())),
        )));

        Self {
            operation,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl<'a> Display for LeftJoin<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_leftjoin(self))
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for LeftJoin<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("LeftJoin next()");

        self.operation.next()
    }
}

impl<O> Selectivity for LeftJoin<O> {}
