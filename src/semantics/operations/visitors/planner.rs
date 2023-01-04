use crate::{
    semantics::operations::{
        filter::Filter,
        join::{CollJoin, IterJoin, Join, NewJoin},
        leftjoin::LeftJoin,
        limit::{CollLimit, IterLimit, Limit, NewLimit},
        minus::{CollMinus, IterMinus, Minus, NewMinus},
        offset::Offset,
        projection::Projection,
        scan::{CollScan, IterScan, NewScan, Scan},
        union::Union,
        Operation,
    },
    syntax::{database, query},
};

pub(crate) struct Planner<'a, S, J, M, L> {
    db: &'a database::Database,
    scan: NewScan<'a, S, J, M, L>,
    join: NewJoin<'a, S, J, M, L>,
    minus: NewMinus<'a, S, J, M, L>,
    limit: NewLimit<'a, S, J, M, L>,
}

impl<'a> Planner<'a, IterScan<'a>, IterJoin, IterMinus, IterLimit> {
    pub(crate) fn iter(db: &'a database::Database) -> Self {
        Self {
            db,
            scan: Scan::iterator,
            join: Join::iterator,
            minus: Minus::iterator,
            limit: Limit::iterator,
        }
    }
}

impl<'a> Planner<'a, CollScan, CollJoin, CollMinus, CollLimit> {
    pub(crate) fn coll(db: &'a database::Database) -> Self {
        Self {
            db,
            scan: Scan::collection,
            join: Join::collection,
            minus: Minus::collection,
            limit: Limit::collection,
        }
    }
}

impl<'a, S, J, M, L> query::QueryVisitor<'a, Operation<'a, S, J, M, L>> for Planner<'a, S, J, M, L>
where
    S: Clone,
    J: Clone,
    M: Clone,
    L: Clone,
{
    fn visit_select(
        &mut self,
        vars: &'a query::Variables,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a, S, J, M, L> {
        Operation::Projection(Projection::new(
            self.visit_modifier(expr, modifier),
            vars.to_owned(),
        ))
    }

    fn visit_ask(
        &mut self,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a, S, J, M, L> {
        self.visit_modifier(expr, modifier)
    }

    fn visit_spo(
        &mut self,
        subject: &'a query::Subject,
        predicate: &'a query::Predicate,
        object: &'a query::Object,
    ) -> Operation<'a, S, J, M, L> {
        Operation::Scan((self.scan)(
            &self.db,
            subject.to_owned(),
            predicate.to_owned(),
            object.to_owned(),
        ))
    }

    fn visit_and(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a, S, J, M, L> {
        Operation::Join((self.join)(self.visit_expr(left), self.visit_expr(right)))
    }

    fn visit_union(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a, S, J, M, L> {
        Operation::Union(Union::new(self.visit_expr(left), self.visit_expr(right)))
    }

    fn visit_optional(
        &mut self,
        left: &'a query::Expression,
        right: &'a query::Expression,
    ) -> Operation<'a, S, J, M, L> {
        Operation::LeftJoin(LeftJoin::new(
            self.visit_expr(left),
            self.visit_expr(right),
            self.join,
            self.minus,
        ))
    }

    fn visit_filter(
        &mut self,
        expr: &'a query::Expression,
        cond: &'a query::Condition,
    ) -> Operation<'a, S, J, M, L> {
        Operation::Filter(Filter::new(self.visit_expr(expr), cond.to_owned()))
    }

    fn visit_modifier(
        &mut self,
        expr: &'a query::Expression,
        modifier: &'a query::SolutionModifier,
    ) -> Operation<'a, S, J, M, L> {
        let mut result = self.visit_expr(expr);

        if let Some(offset) = modifier.offset {
            result = Operation::Offset(Offset::new(result, offset));
        }

        if let Some(limit) = modifier.limit {
            result = Operation::Limit((self.limit)(result, limit));
        }

        result
    }
}
