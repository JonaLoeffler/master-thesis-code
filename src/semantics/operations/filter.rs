use std::fmt::{self, Display};

use crate::{
    semantics::{
        mapping::Mapping,
        selectivity::{Selectivity, SelectivityResult},
    },
    syntax::{database::Summary, query},
};

use super::{
    visitors::{condition::ConditionInfo, printer::Printer},
    Operation, OperationVisitor,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Filter<O> {
    pub(crate) operation: Box<O>,
    pub(crate) condition: Box<query::Condition>,
}

impl<'a> Filter<Operation<'a>> {
    pub(crate) fn new(operation: Operation<'a>, condition: query::Condition) -> Self {
        Self {
            operation: Box::new(operation),
            condition: Box::new(condition),
        }
    }
}

impl<'a> Display for Filter<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_filter(self))
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Filter<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        self.operation.find(|m| m.satisfies(&self.condition))
    }
}

impl<O: Selectivity> Selectivity for Filter<O> {
    fn sel_vc(&self) -> SelectivityResult {
        self.operation.sel_vc()
    }

    fn sel_vcp(&self) -> SelectivityResult {
        self.operation.sel_vcp()
    }

    fn sel_pf(&self, s: &Summary) -> SelectivityResult {
        self.operation.sel_pf(s)
    }

    fn sel_pfc(&self, s: &Summary, i: &ConditionInfo) -> SelectivityResult {
        self.operation.sel_pfc(s, i)
    }

    fn sel_pfj(&self, s: &Summary) -> SelectivityResult {
        self.operation.sel_pfj(s)
    }

    fn sel_pfjc(&self, s: &Summary, i: &ConditionInfo) -> SelectivityResult {
        self.operation.sel_pfjc(s, i)
    }
}
