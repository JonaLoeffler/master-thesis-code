use itertools::Itertools;
use std::{error::Error, fmt::Display};

use crate::semantics::{
    operations::{
        filter::Filter,
        join::{CollJoin, IterJoin, Join, NewJoin},
        leftjoin::LeftJoin,
        limit::{CollLimit, IterLimit, Limit, NewLimit},
        minus::{CollMinus, IterMinus, Minus, NewMinus},
        offset::Offset,
        projection::Projection,
        scan::{CollScan, IterScan, Scan},
        union::Union,
        Operation, OperationVisitor,
    },
    selectivity::{SelectivityError, SelectivityEstimator},
};

use super::flatten::Flatten;

pub(crate) struct Optimize<'a, S, J, M, L> {
    pub(crate) estimator: SelectivityEstimator<'a>,
    join: NewJoin<'a, S, J, M, L>,
    minus: NewMinus<'a, S, J, M, L>,
    limit: NewLimit<'a, S, J, M, L>,
}

impl<'a> Optimize<'a, CollScan, CollJoin, CollMinus, CollLimit> {
    pub(crate) fn coll(optimizer: SelectivityEstimator<'a>) -> Self {
        Self {
            estimator: optimizer,
            join: Join::collection,
            minus: Minus::collection,
            limit: Limit::collection,
        }
    }
}

impl<'a> Optimize<'a, IterScan<'a>, IterJoin, IterMinus, IterLimit> {
    pub(crate) fn iter(optimizer: SelectivityEstimator<'a>) -> Self {
        Self {
            estimator: optimizer,
            join: Join::iterator,
            minus: Minus::iterator,
            limit: Limit::iterator,
        }
    }
}

type OptimizeResult<'a, S, J, M, L> = Result<Operation<'a, S, J, M, L>, OptimizerError>;
#[derive(Debug)]
pub enum OptimizerError {
    UnexpectedOperation,
    Selectivity(SelectivityError),
}
impl Display for OptimizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizerError::UnexpectedOperation => {
                f.write_str(&format!("Unexpected scan in query"))
            }
            OptimizerError::Selectivity(s) => f.write_str(&format!("Selectivity error: {:?}", s)),
        }
    }
}
impl Error for OptimizerError {}
impl From<SelectivityError> for OptimizerError {
    fn from(e: SelectivityError) -> Self {
        OptimizerError::Selectivity(e)
    }
}

impl<'a, S, J, M, L> OperationVisitor<'a, S, J, M, L, OptimizeResult<'a, S, J, M, L>>
    for Optimize<'a, S, J, M, L>
where
    S: Clone + PartialEq,
    J: Clone + PartialEq,
    M: Clone + PartialEq,
    L: Clone + PartialEq,
{
    fn visit(&mut self, o: &'a Operation<'a, S, J, M, L>) -> OptimizeResult<'a, S, J, M, L> {
        if let SelectivityEstimator::Off = self.estimator {
            return Ok(o.to_owned());
        }

        if let Ok(ops) = Flatten::new().visit(o) {
            let mut scans: Vec<(Scan<'a, S, J, M, L>, f32)> = ops
                .iter()
                .map(|o| match o {
                    Operation::Scan(s) => Ok((s.clone(), self.estimator.selectivity(Box::new(s))?)),
                    _ => Err(OptimizerError::UnexpectedOperation),
                })
                .collect::<Result<Vec<(Scan<'a, S, J, M, L>, f32)>, OptimizerError>>()?;

            // No point in optimizing less than 2 scans...
            if scans.len() == 1 {
                return Ok(Operation::Scan(scans.first().unwrap().0.to_owned()));
            }

            scans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for scan in scans.iter() {
                log::debug!(
                    "Selectivity ({}) for scan: {:?}\n{}\n",
                    self.estimator,
                    scan.1,
                    scan.0
                );
            }

            let mut joins: Vec<(Join<J, Operation<'a, S, J, M, L>>, f32)> = ops
                .iter()
                .cartesian_product(ops.iter())
                .filter(|(a, b)| a != b)
                .map(|(a, b)| (self.join)(a.to_owned(), b.to_owned()))
                .filter(|j| !j.join_vars().is_empty())
                .map(|j| Ok((j.clone(), self.estimator.selectivity(Box::new(&j))?)))
                .collect::<Result<Vec<(Join<J, Operation<'a, S, J, M, L>>, f32)>, OptimizerError>>(
                )?;

            joins.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for join in joins.iter() {
                log::debug!(
                    "Selectivity ({}) for join: {:?}\n{}\n",
                    self.estimator,
                    join.1,
                    join.0
                );
            }

            let mut visited: Vec<Box<Operation<'a, S, J, M, L>>> = Vec::new();

            let mut first = joins.first().unwrap().0.to_owned();
            if self.estimator.selectivity(Box::new(&*first.left)).unwrap()
                > self.estimator.selectivity(Box::new(&*first.right)).unwrap()
            {
                first = (self.join)(*first.right, *first.left)
            }

            log::debug!(
                "Picked first join with selectivity ({}): {:?}\n{}\n",
                self.estimator,
                self.estimator.selectivity(Box::new(&first)).unwrap(),
                first
            );

            visited.push(first.left.to_owned());
            visited.push(first.right.to_owned());

            let mut plan: Operation<'a, S, J, M, L> = Operation::Join(first);

            // - Select join with the lowest lowest selectivity value and lower selectivity left
            //   operation
            // - Mark scans of first join as visited
            // - While not all scans are visited
            //     - Add scan that satisfies
            //       * one scan of that join is already visited
            //       * has the lowest selectivity of the
            while visited.len() < scans.len() {
                if let Some(join) = joins.iter().find(|j| {
                    (visited.contains(&j.0.left))
                        && !(visited.contains(&j.0.left) && visited.contains(&j.0.right))
                }) {
                    plan = Operation::Join((self.join)(*join.0.right.clone(), plan));

                    visited.push(join.0.right.to_owned());

                    log::debug!(
                        "Added right operation with selectivity ({}): {:?}\n{}\n",
                        self.estimator,
                        join.1,
                        plan
                    );
                }

                if let Some(join) = joins.iter().find(|j| {
                    (visited.contains(&j.0.right))
                        && !(visited.contains(&j.0.left) && visited.contains(&j.0.right))
                }) {
                    plan = Operation::Join((self.join)(plan, *join.0.left.clone()));

                    visited.push(join.0.left.to_owned());

                    log::debug!(
                        "Added left operation with selectivity ({}): {:?}\n{}\n",
                        self.estimator,
                        join.1,
                        plan
                    );
                }
            }

            return Ok(plan);
        }

        match o {
            Operation::Scan(s) => self.visit_scan(s),
            Operation::Join(j) => self.visit_join(j),
            Operation::Projection(p) => self.visit_projection(p),
            Operation::Union(u) => self.visit_union(u),
            Operation::Filter(f) => self.visit_filter(f),
            Operation::LeftJoin(l) => self.visit_leftjoin(l),
            Operation::Minus(m) => self.visit_minus(m),
            Operation::Offset(o) => self.visit_offset(o),
            Operation::Limit(l) => self.visit_limit(l),
        }
    }

    fn visit_scan(&mut self, _: &'a Scan<'a, S, J, M, L>) -> OptimizeResult<'a, S, J, M, L> {
        panic!("Should have optimized before now")
    }

    fn visit_join(
        &mut self,
        _: &'a Join<J, Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        panic!("Should have optimized before now")
    }

    fn visit_projection(
        &mut self,
        o: &'a Projection<Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Projection(Projection::new(
            self.visit(&o.operation)?,
            o.vars.to_owned(),
        )))
    }

    fn visit_union(
        &mut self,
        o: &'a Union<Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Union(Union::new(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
        )))
    }

    fn visit_filter(
        &mut self,
        o: &'a Filter<Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Filter(Filter::new(
            self.visit(&o.operation)?,
            *o.condition.to_owned(),
        )))
    }

    fn visit_leftjoin(
        &mut self,
        o: &'a LeftJoin<Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::LeftJoin(LeftJoin::new(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
            self.join,
            self.minus,
        )))
    }

    fn visit_minus(
        &mut self,
        o: &'a Minus<M, Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Minus((self.minus)(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
        )))
    }

    fn visit_offset(
        &mut self,
        o: &'a Offset<Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Offset(Offset::new(
            self.visit(&o.operation)?,
            o.offset,
        )))
    }

    fn visit_limit(
        &mut self,
        o: &'a crate::semantics::operations::limit::Limit<L, Operation<'a, S, J, M, L>>,
    ) -> OptimizeResult<'a, S, J, M, L> {
        Ok(Operation::Limit((self.limit)(
            self.visit(&o.operation)?,
            o.limit,
        )))
    }
}
