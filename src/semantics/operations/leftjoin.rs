use std::fmt::{self, Display};

use crate::semantics::{
    mapping::{Mapping, MappingSet},
    selectivity::Selectivity,
};

use super::{join::NewJoin, minus::NewMinus, union::Union, Execute, Operation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LeftJoin<O> {
    pub(crate) operation: Box<O>,
    pub(crate) left: Box<O>,
    pub(crate) right: Box<O>,
}

impl<'a, S, J, M, L> LeftJoin<Operation<'a, S, J, M, L>>
where
    S: Clone,
    J: Clone,
    M: Clone,
    L: Clone,
{
    pub(crate) fn new(
        left: Operation<'a, S, J, M, L>,
        right: Operation<'a, S, J, M, L>,
        join: NewJoin<'a, S, J, M, L>,
        minus: NewMinus<'a, S, J, M, L>,
    ) -> Self {
        let operation = Box::new(Operation::Union(Union::new(
            Operation::Join((join)(left.clone(), right.clone())),
            Operation::Minus((minus)(left.clone(), right.clone())),
        )));

        Self {
            operation,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl<O: Display> Display for LeftJoin<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("LEFTJOIN"))?;

        f.write_str(&format!("\n{}", self.operation).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O: Execute> Execute for LeftJoin<O> {
    fn execute(&self) -> MappingSet {
        self.operation.execute()
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
