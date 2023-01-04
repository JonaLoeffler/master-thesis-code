use std::collections::HashMap;

use crate::syntax::query::{
    Condition, Expression, Object, Predicate, Query, SolutionModifier, Subject, Type, Variables,
};

pub fn example1() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?e".into(), "?a".into()]),
            Expression::And(
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<email>".into()),
                    object: Object::V("?e".into()),
                }),
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<age>".into()),
                    object: Object::V("?a".into()),
                }),
            ),
            SolutionModifier::default(),
        ),
    }
}

pub fn example2() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?a".into()]),
            Expression::Triple {
                subject: Subject::V("?p".into()),
                predicate: Predicate::I("<age>".into()),
                object: Object::V("?a".into()),
            },
            SolutionModifier::default(),
        ),
    }
}

pub fn example3() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::AskQuery(
            Expression::Triple {
                subject: Subject::V("?p".into()),
                predicate: Predicate::I("<age>".into()),
                object: Object::V("?a".into()),
            },
            SolutionModifier::default(),
        ),
    }
}

pub fn example4() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?a".into()]),
            Expression::Triple {
                subject: Subject::I("<P1>".into()),
                predicate: Predicate::I("<age>".into()),
                object: Object::V("?a".into()),
            },
            SolutionModifier::default(),
        ),
    }
}

pub fn example5() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?a".into()]),
            Expression::Filter(
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<age>".into()),
                    object: Object::V("?a".into()),
                }),
                Box::new(Condition::Or(
                    Box::new(Condition::Equals(
                        Object::V("?a".into()),
                        Object::L("30".into()),
                    )),
                    Box::new(Condition::Equals(
                        Object::V("?a".into()),
                        Object::L("26".into()),
                    )),
                )),
            ),
            SolutionModifier::default(),
        ),
    }
}

pub fn example6() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?a".into()]),
            Expression::Triple {
                subject: Subject::V("?p".into()),
                predicate: Predicate::I("<age>".into()),
                object: Object::V("?a".into()),
            },
            SolutionModifier::default(),
        ),
    }
}

pub fn example7() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?a".into(), "?e".into()]),
            Expression::Optional(
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<age>".into()),
                    object: Object::V("?a".into()),
                }),
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<email>".into()),
                    object: Object::V("?e".into()),
                }),
            ),
            SolutionModifier::default(),
        ),
    }
}

pub fn example8() -> Query {
    Query {
        prologue: HashMap::new(),
        kind: Type::SelectQuery(
            Variables::new(vec!["?p".into(), "?a".into(), "?e".into()]),
            Expression::Union(
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<age>".into()),
                    object: Object::V("?a".into()),
                }),
                Box::new(Expression::Triple {
                    subject: Subject::V("?p".into()),
                    predicate: Predicate::I("<email>".into()),
                    object: Object::V("?e".into()),
                }),
            ),
            SolutionModifier::default(),
        ),
    }
}
