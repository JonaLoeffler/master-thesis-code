use core::fmt::{self, Display};

use crate::{
    semantics::{mapping::Mapping, selectivity},
    syntax::{database, query},
};

use super::Execute;

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl<O: Display> Display for Projection<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("PROJECTION {}", self.vars))?;

        f.write_str(&format!("\n{}", self.operation).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<O> Execute for Projection<O>
where
    O: Execute + Display,
{
    fn execute(&self) -> crate::semantics::mapping::MappingSet {
        let results = self.operation.execute();

        log::debug!("Selecting {} from {} mappings", self.vars, results.len());

        results
            .into_iter()
            .map(|m| {
                let mut result = Mapping::new();

                for (i, var) in (&self.vars).iter().enumerate() {
                    result.insert(
                        var.to_owned().set_pos(i),
                        if let Some(o) = m.get(var) {
                            o.to_owned()
                        } else {
                            database::Object::B
                        },
                    );
                }

                result
            })
            .collect()
    }
}

impl<'a, O> Iterator for Projection<O>
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
