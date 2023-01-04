use std::error::Error;

use crate::syntax::{database::Database, query::Query};

use super::options::{EvalOptions, Semantics};

use super::evaluate;
use super::results::QueryResult;

fn iter_evaluate(
    db: &Database,
    query: Query,
    options: Option<EvalOptions>,
) -> Result<QueryResult, Box<dyn Error>> {
    let options = options
        .unwrap_or_default()
        .with_semantics(Semantics::Iterator);

    evaluate(db, query, Some(options))
}

fn coll_evaluate(
    db: &Database,
    query: Query,
    options: Option<EvalOptions>,
) -> Result<QueryResult, Box<dyn Error>> {
    let options = options
        .unwrap_or_default()
        .with_semantics(Semantics::Collection);

    evaluate(db, query, Some(options))
}

mod iterator {
    use super::iter_evaluate as evaluate;
    use crate::examples::databases::example1 as db;
    use crate::examples::queries;
    use crate::semantics::{mapping::Mapping, QueryResult};
    use crate::syntax::database::Object;
    use crate::syntax::query::Variable;

    #[test]
    fn query1() {
        let mapping: Mapping = vec![
            (Variable::new("?p".into()), Object::I("<P3>".into())),
            (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            (Variable::new("?a".into()), Object::L("30".into())),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (v, o))| (v.set_pos(i), o))
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example1(), None).unwrap(),
            QueryResult::select(vec![mapping])
        );
    }

    #[test]
    fn query2() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example2(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query3() {
        assert_eq!(
            evaluate(&db(), queries::example3(), None).unwrap(),
            QueryResult::ask(true)
        );
    }

    #[test]
    fn query4() {
        let mappings = vec![vec![(Variable::new("?a".into()), Object::L("30".into()))]]
            .into_iter()
            .map(|m| {
                m.into_iter()
                    .enumerate()
                    .map(|(i, (v, o))| (v.set_pos(i), o))
                    .collect::<Mapping>()
            })
            .collect();

        assert_eq!(
            evaluate(&db(), queries::example4(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query5() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example5(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query6() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example6(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query7() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
                (Variable::new("?e".into()), Object::B),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example7(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query8() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::B),
                (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example8(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }
}

mod collection {
    use super::coll_evaluate as evaluate;
    use crate::examples::databases::example1 as db;
    use crate::examples::queries;
    use crate::semantics::{mapping::Mapping, QueryResult};
    use crate::syntax::database::Object;
    use crate::syntax::query::Variable;

    #[test]
    fn query1() {
        let mapping: Mapping = vec![
            (Variable::new("?p".into()), Object::I("<P3>".into())),
            (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            (Variable::new("?a".into()), Object::L("30".into())),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (v, o))| (v.set_pos(i), o))
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example1(), None).unwrap(),
            QueryResult::select(vec![mapping])
        );
    }

    #[test]
    fn query2() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example2(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query3() {
        assert_eq!(
            evaluate(&db(), queries::example3(), None).unwrap(),
            QueryResult::ask(true)
        );
    }

    #[test]
    fn query4() {
        let mappings = vec![vec![(Variable::new("?a".into()), Object::L("30".into()))]]
            .into_iter()
            .map(|m| {
                m.into_iter()
                    .enumerate()
                    .map(|(i, (v, o))| (v.set_pos(i), o))
                    .collect::<Mapping>()
            })
            .collect();

        assert_eq!(
            evaluate(&db(), queries::example4(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query5() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example5(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query6() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example6(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query7() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
                (Variable::new("?e".into()), Object::B),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example7(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }

    #[test]
    fn query8() {
        let mappings = vec![
            vec![
                (Variable::new("?p".into()), Object::I("<P1>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P2>".into())),
                (Variable::new("?a".into()), Object::L("29".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::L("30".into())),
                (Variable::new("?e".into()), Object::B),
            ],
            vec![
                (Variable::new("?p".into()), Object::I("<P3>".into())),
                (Variable::new("?a".into()), Object::B),
                (Variable::new("?e".into()), Object::L("joe@tld.com".into())),
            ],
        ]
        .into_iter()
        .map(|m| {
            m.into_iter()
                .enumerate()
                .map(|(i, (v, o))| (v.set_pos(i), o))
                .collect::<Mapping>()
        })
        .collect();

        assert_eq!(
            evaluate(&db(), queries::example8(), None).unwrap(),
            QueryResult::select(mappings)
        );
    }
}
