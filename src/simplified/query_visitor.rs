use super::query_ast::{
    Condition, Expression, Modifier, Object, Predicate, Query, QueryType, Subject, Variable,
};

pub trait QueryVisitor<'a, T> {
    fn visit(&mut self, o: &'a Query) -> T {
        match &o.kind {
            QueryType::SelectQuery(v, e, m) => self.visit_select(v, e, m),
            QueryType::AskQuery(e, m) => self.visit_ask(e, m),
        }
    }

    fn visit_select(
        &mut self,
        vars: &'a Vec<Variable>,
        expr: &'a Expression,
        modi: &'a Modifier,
    ) -> T;

    fn visit_ask(&mut self, expr: &'a Expression, modi: &'a Modifier) -> T;

    fn visit_expr(&mut self, o: &'a Expression) -> T {
        match o {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => self.visit_spo(subject, predicate, object),
            Expression::And(left, right) => self.visit_and(left, right),
            Expression::Union(left, right) => self.visit_union(left, right),
            Expression::Optional(left, right) => self.visit_optional(left, right),
            Expression::Filter(expr, cond) => self.visit_filter(expr, cond),
        }
    }

    fn visit_spo(
        &mut self,
        subject: &'a Subject,
        predicate: &'a Predicate,
        object: &'a Object,
    ) -> T;
    fn visit_and(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_union(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_optional(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_filter(&mut self, expr: &'a Expression, cond: &'a Condition) -> T;
    fn visit_modifier(&mut self, expr: &'a Expression, modi: &'a Modifier) -> T;
}
