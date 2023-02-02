use crate::syntax::database;

use super::query_ast::{Condition, Object, Predicate, Subject, Variable};

enum Operation<'a> {
    Scan(Scan<'a>),
    Join(Join<'a>),
    Projection(Projection<'a>),
    Union(Union<'a>),
    Filter(Filter<'a>),
    LeftJoin(LeftJoin<'a>),
    Minus(Minus<'a>),
    Offset(Offset<'a>),
    Limit(Limit<'a>),
}

struct Scan<'a> {
    db: &'a database::Database,
    subject: Subject,
    predicate: Predicate,
    object: Object,
    // additional state for iterator implementation omitted
}

struct Join<'a> {
    left: Box<Operation<'a>>,
    right: Box<Operation<'a>>,
    join_vars: Vec<Variable>,
    // additional state for iterator implementation omitted
}

struct Projection<'a> {
    operation: Box<Operation<'a>>,
    vars: Vec<Variable>,
}

struct Union<'a> {
    left: Box<Operation<'a>>,
    right: Box<Operation<'a>>,
}

struct Filter<'a> {
    operation: Box<Operation<'a>>,
    condition: Condition,
}

struct LeftJoin<'a> {
    left: Box<Operation<'a>>,
    right: Box<Operation<'a>>,
}

struct Minus<'a> {
    left: Box<Operation<'a>>,
    right: Box<Operation<'a>>,
}

struct Limit<'a> {
    operation: Box<Operation<'a>>,
    limit: usize,
}

struct Offset<'a> {
    operation: Box<Operation<'a>>,
    offset: usize,
}

trait OperationVisitor<'a, R> {
    fn visit(&mut self, o: &'a Operation) -> R {
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

    fn visit_scan(&mut self, o: &'a Scan) -> R;
    fn visit_join(&mut self, o: &'a Join) -> R;
    fn visit_projection(&mut self, o: &'a Projection) -> R;
    fn visit_union(&mut self, o: &'a Union) -> R;
    fn visit_filter(&mut self, o: &'a Filter) -> R;
    fn visit_leftjoin(&mut self, o: &'a LeftJoin) -> R;
    fn visit_minus(&mut self, o: &'a Minus) -> R;
    fn visit_offset(&mut self, o: &'a Offset) -> R;
    fn visit_limit(&mut self, o: &'a Limit) -> R;
}
