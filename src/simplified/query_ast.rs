use std::collections::HashMap;

pub struct Variable {
    name: String,
}

pub enum Subject {
    U(String),
    V(Variable),
}

pub enum Predicate {
    U(String),
    V(Variable),
}

pub enum Object {
    L(String),
    U(String),
    V(Variable),
}

pub struct Query {
    prologue: HashMap<String, String>,
    pub kind: QueryType,
}

pub enum QueryType {
    SelectQuery(Vec<Variable>, Expression, Modifier),
    AskQuery(Expression, Modifier),
}

pub enum Expression {
    Triple {
        subject: Subject,
        predicate: Predicate,
        object: Object,
    },
    And(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Optional(Box<Expression>, Box<Expression>),
    Filter(Box<Expression>, Box<Condition>),
}

pub struct Modifier {
    limit: Option<usize>,
    offset: Option<usize>,
}

pub enum Condition {
    Equals(Object, Object),
    Bound(Variable),
    Not(Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}
