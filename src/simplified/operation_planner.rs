use crate::syntax::database;

use super::{
    operation::Operation,
    query_ast::{Condition, Expression, Modifier, Object, Predicate, Subject, Variable},
    query_visitor::{ExpressionVisitor, QueryVisitor},
};

struct Planner<'a> {
    db: &'a database::Database,
}

impl<'a> QueryVisitor<'a, Operation<'a>> for Planner<'a> {
    fn visit_select(
        &mut self,
        v: &'a Vec<Variable>,
        e: &'a Expression,
        m: &'a Modifier,
    ) -> Operation<'a> {
        todo!()
    }

    fn visit_ask(&mut self, e: &'a Expression, m: &'a Modifier) -> Operation<'a> {
        todo!()
    }

    fn visit_modifier(&mut self, e: &'a Expression, m: &'a Modifier) -> Operation<'a> {
        todo!()
    }
}

impl<'a> ExpressionVisitor<'a, Operation<'a>> for Planner<'a> {
    fn visit_spo(
        &mut self,
        subject: &'a Subject,
        predicate: &'a Predicate,
        object: &'a Object,
    ) -> Operation<'a> {
        todo!()
    }

    fn visit_and(&mut self, left: &'a Expression, right: &'a Expression) -> Operation<'a> {
        todo!()
    }

    fn visit_union(&mut self, left: &'a Expression, right: &'a Expression) -> Operation<'a> {
        todo!()
    }

    fn visit_optional(&mut self, left: &'a Expression, right: &'a Expression) -> Operation<'a> {
        todo!()
    }

    fn visit_filter(&mut self, expr: &'a Expression, cond: &'a Condition) -> Operation<'a> {
        todo!()
    }
}
