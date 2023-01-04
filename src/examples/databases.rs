use crate::syntax::database::{Database, Object, Predicate, Subject, Triple};

pub fn example1() -> Database {
    vec![
        Triple::new(
            Subject::I("<P1>".into()),
            Predicate::I("<age>".into()),
            Object::L("30".into()),
        ),
        Triple::new(
            Subject::I("<P2>".into()),
            Predicate::I("<age>".into()),
            Object::L("29".into()),
        ),
        Triple::new(
            Subject::I("<P3>".into()),
            Predicate::I("<age>".into()),
            Object::L("30".into()),
        ),
        Triple::new(
            Subject::I("<P3>".into()),
            Predicate::I("<email>".into()),
            Object::L("joe@tld.com".into()),
        ),
    ]
    .into_iter()
    .collect::<Database>()
}
