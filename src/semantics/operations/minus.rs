use std::fmt;

use crate::semantics::{
    mapping::{Mapping, MappingSet},
    selectivity::Selectivity,
};

use super::{Execute, Operation};

pub(crate) type NewMinus<'a, S, J, M, L> =
    fn(Operation<'a, S, J, M, L>, Operation<'a, S, J, M, L>) -> Minus<M, Operation<'a, S, J, M, L>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Minus<M, O> {
    pub(super) left: Box<O>,
    pub(super) right: Box<O>,
    kind: M,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CollMinus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IterMinus {
    collected: Vec<Mapping>,
}

impl<'a, S, J, M, L> Minus<IterMinus, Operation<'a, S, J, M, L>> {
    pub(crate) fn iterator(
        left: Operation<'a, S, J, M, L>,
        right: Operation<'a, S, J, M, L>,
    ) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            kind: IterMinus { collected: vec![] },
        }
    }
}

impl<'a, S, J, M, L> Minus<CollMinus, Operation<'a, S, J, M, L>> {
    pub(crate) fn collection(
        left: Operation<'a, S, J, M, L>,
        right: Operation<'a, S, J, M, L>,
    ) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            kind: CollMinus,
        }
    }
}

impl<'a, M, O: fmt::Display> fmt::Display for Minus<M, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("MINUS"))?;

        f.write_str(&format!("\n{}", self.left).replace("\n", "\n  "))?;
        f.write_str(&format!("\n{}", self.right).replace("\n", "\n  "))?;

        Ok(())
    }
}

impl<'a, O: Execute> Execute for Minus<CollMinus, O> {
    fn execute(&self) -> MappingSet {
        let left = self.left.execute();
        let right = self.right.execute();

        left.into_iter()
            .filter(|next| {
                right
                    .iter()
                    .map(|mr| !next.compatible(&mr))
                    .reduce(|accum, item| accum && item)
                    .unwrap_or(true)
            })
            .collect()
    }
}

impl<'a, O: Iterator<Item = Mapping>> Iterator for Minus<IterMinus, O> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("Minus next()");

        if self.kind.collected.is_empty() {
            log::debug!("Building rejection list");
        }

        while let Some(next) = self.right.next() {
            self.kind.collected.push(next);
        }

        log::trace!("Minus next() with {} to filter", self.kind.collected.len());

        while let Some(next) = self.left.next() {
            let compatible = self
                .kind
                .collected
                .iter()
                .map(|mr| !next.compatible(&mr))
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

impl<M, O> Selectivity for Minus<M, O> {}
