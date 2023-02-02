use crate::{
    semantics::operations::{
        filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, offset::Offset,
        projection::Projection, scan::Scan, union::Union, Operation,
    },
    syntax::{
        database,
        query::{self, ConditionVisitor, ExpressionVisitor},
    },
};

use super::condition::Normalize;

pub(crate) struct Planner<'a> {
    db: &'a database::Database,
}

impl<'a> Planner<'a> {
    pub(crate) fn new(db: &'a database::Database) -> Self {
        Self { db }
    }
}

impl<'a> query::QueryVisitor<'a, Operation<'a>> for Planner<'a> {
    fn visit_select(
        &mut self,
        vars: &'a query::Variables,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a> {
        Operation::Projection(Projection::new(
            self.visit_modifier(expr, modifier),
            vars.to_owned(),
        ))
    }

    fn visit_ask(
        &mut self,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a> {
        self.visit_modifier(expr, modifier)
    }

    fn visit_modifier(
        &mut self,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a> {
        let mut result = ExpressionVisitor::visit(self, expr);

        if let Some(offset) = modifier.offset {
            result = Operation::Offset(Offset::new(result, offset));
        }

        if let Some(limit) = modifier.limit {
            result = Operation::Limit(Limit::new(result, limit));
        }

        result
    }
}

impl<'a> ExpressionVisitor<'a, Operation<'a>> for Planner<'a> {
    fn visit_spo(
        &mut self,
        subject: &'a query::Subject,
        predicate: &'a query::Predicate,
        object: &'a query::Object,
    ) -> Operation<'a> {
        Operation::Scan(Scan::new(
            self.db,
            subject.to_owned(),
            predicate.to_owned(),
            object.to_owned(),
        ))
    }

    fn visit_and(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a> {
        Operation::Join(Join::new(self.visit(left), self.visit(right)))
    }

    fn visit_union(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a> {
        Operation::Union(Union::new(self.visit(left), self.visit(right)))
    }

    fn visit_optional(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a> {
        Operation::LeftJoin(LeftJoin::new(self.visit(left), self.visit(right)))
    }

    fn visit_filter(
        &mut self,
        expr: &'a query::Expression,
        cond: &'a query::Condition,
    ) -> Operation<'a> {
        Operation::Filter(Filter::new(self.visit(expr), Normalize::new().visit(cond)))
    }
}
