use itertools::Itertools;
use std::{error::Error, fmt::Display};

use crate::{
    semantics::{
        operations::{
            filter::Filter,
            join::Join,
            leftjoin::LeftJoin,
            limit::Limit,
            minus::Minus,
            offset::Offset,
            projection::Projection,
            scan::Scan,
            union::Union,
            visitors::{bound::BoundVars, condition::VariableInfo},
            Operation, OperationVisitor,
        },
        selectivity::{SelectivityError, SelectivityEstimator},
    },
    syntax::query::{Condition, ConditionVisitor, Object},
};

use super::{
    condition::{ConditionAnalyzer, ConditionInfo},
    flatten::Flatten,
    printer::Printer,
};

pub(crate) struct Optimize<'a> {
    pub(crate) estimator: SelectivityEstimator<'a>,
    pub(crate) condition: bool,

    printer: Printer<'a>,

    condition_info: ConditionInfo,
}

impl<'a> Optimize<'a> {
    pub(crate) fn with_condition(self, condition: bool) -> Self {
        Self {
            estimator: self.estimator,
            condition,

            printer: self.printer,

            condition_info: ConditionInfo::new(),
        }
    }
}

impl<'a> Optimize<'a> {
    pub(crate) fn new(optimizer: SelectivityEstimator<'a>) -> Self {
        Self {
            estimator: optimizer.clone(),
            condition: false,

            printer: Printer::new()
                .with_estimator(Some(optimizer.clone()))
                .with_bgp(true)
                .with_join(false)
                .with_bound(false),

            condition_info: ConditionInfo::new(),
        }
    }
}

type OptimizeResult<'a> = Result<Operation<'a>, OptimizerError>;
#[derive(Debug)]
pub enum OptimizerError {
    UnexpectedOperation,
    Selectivity(SelectivityError),
}
impl Display for OptimizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizerError::UnexpectedOperation => f.write_str("Unexpected scan in query"),
            OptimizerError::Selectivity(e) => f.write_str(&format!("Selectivity error: {e:?}")),
        }
    }
}
impl Error for OptimizerError {}
impl From<SelectivityError> for OptimizerError {
    fn from(e: SelectivityError) -> Self {
        OptimizerError::Selectivity(e)
    }
}

impl<'a> OperationVisitor<'a, OptimizeResult<'a>> for Optimize<'a> {
    fn visit(&mut self, o: &'a Operation<'a>) -> OptimizeResult<'a> {
        if let SelectivityEstimator::Off = self.estimator {
            return Ok(o.to_owned());
        }

        if let Ok(ops) = Flatten::new().visit(o) {
            let mut scans: Vec<(Scan<'a>, f64)> = ops
                .iter()
                .map(|o| match o {
                    Operation::Scan(s) => Ok((s.clone(), self.estimator.selectivity(s)?)),
                    _ => Err(OptimizerError::UnexpectedOperation),
                })
                .collect::<Result<Vec<(Scan<'a>, f64)>, OptimizerError>>()?;

            // No point in optimizing less than 2 scans...
            if scans.len() == 1 {
                return Ok(Operation::Scan(scans.first().unwrap().0.to_owned()));
            }

            scans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            log::info!("Selectivities of {} Scans: ", scans.len());
            log::info!(" --- SCANS --- ");
            for scan in scans.iter() {
                log::info!(
                    "Estimating selectivity {:1.2e}\n{}\n",
                    scan.1,
                    self.printer.visit_scan(&scan.0)
                );
            }
            log::info!(" --- SCANS --- ");

            let mut joins: Vec<(Join<Operation<'a>>, f64)> = ops
                .iter()
                .cartesian_product(ops.iter())
                .filter(|(a, b)| a != b)
                .map(|(a, b)| Join::new(a.to_owned(), b.to_owned()))
                // .filter(|j| !j.join_vars.is_empty())
                .map(|j| Ok((j.clone(), self.estimator.selectivity(&j)?)))
                .collect::<Result<Vec<(Join<Operation<'a>>, f64)>, OptimizerError>>()?
                .into_iter()
                .map(|(j, s)| {
                    (
                        Join::new(
                            insert_filter_operation(*j.left, &self.condition_info),
                            insert_filter_operation(*j.right, &self.condition_info),
                        ),
                        s,
                    )
                })
                .collect();

            joins.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            log::info!("Selectivities of {} Joins: ", joins.len());
            log::info!(" --- JOINS --- ");
            for join in joins.iter() {
                log::info!(
                    "Estimated selectivity {:1.2e} for JOIN, will return {} \n{}\n",
                    join.1,
                    f32::NAN,
                    // join.0.clone().count(),
                    self.printer.visit_join(&join.0)
                );
            }
            log::info!(" --- JOINS --- ");

            let mut visited: Vec<Box<Operation<'a>>> = Vec::new();

            let mut first = joins
                .iter()
                .cloned()
                .map(|(j, _)| j)
                .find(|j| !j.join_vars.is_empty())
                .unwrap_or(joins.first().unwrap().0.clone());

            if self.estimator.selectivity(&*first.left).unwrap()
                > self.estimator.selectivity(&*first.right).unwrap()
            {
                first = Join::new(*first.right, *first.left)
            }

            log::debug!(
                "Picked first join with selectivity ({}): {:?}\n{}\n",
                self.estimator,
                self.estimator.selectivity(&first).unwrap_or_default(),
                first
            );

            visited.push(first.left.to_owned());
            visited.push(first.right.to_owned());

            let mut plan: Operation<'a> = Operation::Join(first);

            // - Select join with the lowest lowest selectivity value and lower selectivity left
            //   operation
            // - Mark scans of first join as visited
            // - While not all scans are visited
            //     - Add scan that satisfies
            //       * one scan of that join is already visited
            //       * has the lowest selectivity of the remaining
            while visited.len() < scans.len() {
                if let Some(join) = joins.iter().find(|j| {
                    !j.0.join_vars.is_empty()
                        && visited.contains(&j.0.left)
                        && !(visited.contains(&j.0.left) && visited.contains(&j.0.right))
                }) {
                    plan = Operation::Join(Join::new(*join.0.right.clone(), plan));

                    visited.push(join.0.right.to_owned());

                    log::debug!(
                        "Added right operation with selectivity ({}): {:?}\n{}\n",
                        self.estimator,
                        join.1,
                        self.printer.visit(&plan)
                    );

                    continue;
                }

                if let Some(join) = joins.iter().find(|j| {
                    !j.0.join_vars.is_empty()
                        && visited.contains(&j.0.right)
                        && !(visited.contains(&j.0.left) && visited.contains(&j.0.right))
                }) {
                    plan = Operation::Join(Join::new(plan, *join.0.left.clone()));

                    visited.push(join.0.left.to_owned());

                    log::debug!(
                        "Added left operation with selectivity ({}): {:?}\n{}\n",
                        self.estimator,
                        join.1,
                        self.printer.visit(&plan)
                    );

                    continue;
                }

                let join = joins.first().unwrap();
                plan = Operation::Join(Join::new(*join.0.right.clone(), plan));
                visited.push(join.0.right.to_owned());

                log::debug!(
                    "Added disjunct BGP operation with selectivity ({}): {:?}\n{}\n",
                    self.estimator,
                    join.1,
                    self.printer.visit(&plan)
                );
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

    fn visit_scan(&mut self, _: &'a Scan<'a>) -> OptimizeResult<'a> {
        panic!("Should have optimized before now")
    }

    fn visit_join(&mut self, _: &'a Join<Operation<'a>>) -> OptimizeResult<'a> {
        panic!("Should have optimized before now")
    }

    fn visit_projection(&mut self, o: &'a Projection<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::Projection(Projection::new(
            self.visit(&o.operation)?,
            o.vars.to_owned(),
        )))
    }

    fn visit_union(&mut self, o: &'a Union<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::Union(Union::new(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
        )))
    }

    fn visit_filter(&mut self, o: &'a Filter<Operation<'a>>) -> OptimizeResult<'a> {
        if self.condition {
            self.condition_info = self
                .condition_info
                .union(ConditionAnalyzer::new().visit(&o.condition));
        }

        Ok(Operation::Filter(Filter::new(
            self.visit(&o.operation)?,
            *o.condition.to_owned(),
        )))
    }

    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::LeftJoin(LeftJoin::new(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
        )))
    }

    fn visit_minus(&mut self, o: &'a Minus<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::Minus(Minus::new(
            self.visit(&o.left)?,
            self.visit(&o.right)?,
        )))
    }

    fn visit_offset(&mut self, o: &'a Offset<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::Offset(Offset::new(
            self.visit(&o.operation)?,
            o.offset,
        )))
    }

    fn visit_limit(&mut self, o: &'a Limit<Operation<'a>>) -> OptimizeResult<'a> {
        Ok(Operation::Limit(Limit::new(
            self.visit(&o.operation)?,
            o.limit,
        )))
    }
}

fn insert_filter_operation<'a>(op: Operation<'a>, info: &ConditionInfo) -> Operation<'a> {
    let operation = op.clone();
    let mut condition = None;

    for v in BoundVars::new().visit(&operation) {
        if let Some(infos) = info.get(&v) {
            for info in infos.iter() {
                let next = match info {
                    VariableInfo::Lt(l) => {
                        Condition::LT(Object::V(v.to_owned()), Object::L(l.to_owned()))
                    }
                    VariableInfo::Gt(l) => {
                        Condition::GT(Object::V(v.to_owned()), Object::L(l.to_owned()))
                    }
                    VariableInfo::Lte(l) => Condition::Not(Box::new(Condition::GT(
                        Object::V(v.to_owned()),
                        Object::L(l.to_owned()),
                    ))),
                    VariableInfo::Gte(l) => Condition::Not(Box::new(Condition::LT(
                        Object::V(v.to_owned()),
                        Object::L(l.to_owned()),
                    ))),
                    VariableInfo::EqualsIri(i) => {
                        Condition::Equals(Object::V(v.to_owned()), Object::I(i.to_owned()))
                    }
                    VariableInfo::EqualsLiteral(l) => {
                        Condition::Equals(Object::V(v.to_owned()), Object::L(l.to_owned()))
                    }
                    VariableInfo::NotEqualsLiteral(l) => Condition::Not(Box::new(
                        Condition::Equals(Object::V(v.to_owned()), Object::L(l.to_owned())),
                    )),
                    VariableInfo::NotEqualsIri(i) => Condition::Not(Box::new(Condition::Equals(
                        Object::V(v.to_owned()),
                        Object::I(i.to_owned()),
                    ))),
                    VariableInfo::Bound => Condition::Bound(v.to_owned()),
                    VariableInfo::UnBound => {
                        Condition::Not(Box::new(Condition::Bound(v.to_owned())))
                    }
                };

                condition = if let Some(c) = condition {
                    Some(Condition::And(Box::new(c), Box::new(next)))
                } else {
                    Some(next)
                }
            }
        }
    }

    match condition {
        Some(c) => Operation::Filter(Filter::new(operation, c)),
        None => operation,
    }
}
