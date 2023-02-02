use std::fmt::{self, Display};
use std::hash::Hash;

use crate::semantics::{mapping::Mapping, selectivity::Selectivity};

use super::{visitors::printer::Printer, Operation, OperationVisitor};

#[derive(Debug, Clone)]
pub(crate) struct Minus<O> {
    pub(super) left: Box<O>,
    pub(super) right: Box<O>,
    collected: Vec<Mapping>,
}

impl<'a> Minus<Operation<'a>> {
    pub(crate) fn new(left: Operation<'a>, right: Operation<'a>) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            collected: vec![],
        }
    }
}

impl<'a> fmt::Display for Minus<Operation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit_minus(self))
    }
}

impl<O: Hash + Display> Hash for Minus<O> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.left.hash(state);
        self.right.hash(state);
    }
}

impl<O: PartialEq> Eq for Minus<O> {}
impl<O: PartialEq> PartialEq for Minus<O> {
    fn eq(&self, other: &Self) -> bool {
        self.left.eq(&other.left) && self.right.eq(&other.right)
    }
}

impl<O: Iterator<Item = Mapping>> Iterator for Minus<O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Minus next()");

        if self.collected.is_empty() {
            log::debug!("Building rejection list");
        }

        for next in self.right.by_ref() {
            self.collected.push(next);
        }

        log::trace!("Minus next() with {} to filter", self.collected.len());

        for next in self.left.by_ref() {
            let compatible = self
                .collected
                .iter()
                .map(|mr| !next.compatible(mr))
                .reduce(|accum, item| accum && item)
                .unwrap_or(true);

            if compatible {
                log::trace!("Minus next() returns {next}");

                return Some(next);
            };
        }

        log::trace!("Minus next() returns None");

        None
    }
}

impl<O> Selectivity for Minus<O> {}
