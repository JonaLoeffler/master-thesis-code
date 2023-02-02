use core::fmt::{self, Display};

use crate::{
    semantics::{mapping::Mapping, selectivity},
    syntax::{database, query},
};

use super::{visitors::printer::Printer, Operation, OperationVisitor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Projection<O: Display> {
    pub(super) operation: Box<O>,
    pub(super) vars: query::Variables,
}

impl<O: Display> Projection<O> {
    pub(crate) fn new(operation: O, vars: query::Variables) -> Self {
        Self {
            operation: Box::new(operation),
            vars,
        }
    }
}

impl<'a> Display for Projection<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_projection(self))
    }
}

impl<O> Iterator for Projection<O>
where
    O: Iterator<Item = Mapping> + Display,
{
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Projection next()");

        if let Some(mapping) = self.operation.next() {
            let mut result = Mapping::new();

            for (i, var) in self.vars.iter().enumerate() {
                result.insert(
                    var.to_owned().set_pos(i),
                    if let Some(o) = mapping.get(var) {
                        o.to_owned()
                    } else {
                        database::Object::B
                    },
                );
            }

            log::trace!("Projection next() returns {result}");

            Some(result)
        } else {
            None
        }
    }
}

impl<O: Display> selectivity::Selectivity for Projection<O> {}
