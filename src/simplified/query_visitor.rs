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

    fn visit_select(&mut self, v: &'a Vec<Variable>, e: &'a Expression, m: &'a Modifier) -> T;
    fn visit_ask(&mut self, e: &'a Expression, m: &'a Modifier) -> T;

    fn visit_modifier(&mut self, e: &'a Expression, m: &'a Modifier) -> T;
}

pub trait ExpressionVisitor<'a, T> {
    fn visit(&mut self, o: &'a Expression) -> T {
        match o {
            Expression::Triple(s, p, o) => self.visit_spo(s, p, o),
            Expression::And(left, right) => self.visit_and(left, right),
            Expression::Union(left, right) => self.visit_union(left, right),
            Expression::Optional(left, right) => self.visit_optional(left, right),
            Expression::Filter(expr, cond) => self.visit_filter(expr, cond),
        }
    }

    fn visit_spo(&mut self, s: &'a Subject, p: &'a Predicate, o: &'a Object) -> T;
    fn visit_and(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_union(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_optional(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_filter(&mut self, expr: &'a Expression, cond: &'a Condition) -> T;
}

pub trait ConditionVisitor<T> {
    fn visit(&mut self, c: &Condition) -> T {
        match c {
            Condition::Equals(o1, o2) => self.visit_equals(o1, o2),
            Condition::LT(o1, o2) => self.visit_lt(o1, o2),
            Condition::GT(o1, o2) => self.visit_gt(o1, o2),
            Condition::Bound(v) => self.visit_bound(v),
            Condition::Not(e) => self.visit_not(e),
            Condition::And(e1, e2) => self.visit_and(e1, e2),
            Condition::Or(e1, e2) => self.visit_or(e1, e2),
        }
    }

    fn visit_equals(&mut self, left: &Object, right: &Object) -> T;
    fn visit_lt(&mut self, left: &Object, right: &Object) -> T;
    fn visit_gt(&mut self, left: &Object, right: &Object) -> T;
    fn visit_bound(&mut self, v: &Variable) -> T;
    fn visit_not(&mut self, c: &Condition) -> T;
    fn visit_and(&mut self, left: &Condition, right: &Condition) -> T;
    fn visit_or(&mut self, left: &Condition, right: &Condition) -> T;
}
