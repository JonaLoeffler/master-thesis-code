use std::collections::HashMap;

use crate::syntax::query::{
    Expression, Object, Predicate, Query, SolutionModifier, Subject, Type::SelectQuery, Variable,
    Variables,
};

pub fn prologue() -> HashMap<String, String> {
    HashMap::from([
        (
            "rdf".into(),
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#>".into(),
        ),
        (
            "ub".into(),
            "<http://swat.cse.lehigh.edu/onto/univ-bench.owl#>".into(),
        ),
    ])
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:GraduateStudent .
///   ?X ub:takesCourse <http://www.Department0.University0.edu/GraduateCourse0>
/// }
/// ```
///
pub fn query1() -> Query {
    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(
                Box::new(Expression::Triple {
                    subject: Subject::V(Variable::new("?X".into())),
                    predicate: Predicate::I("rdf:type".into()),
                    object: Object::I("ub:GraduateStudent".into()),
                }),
                Box::new(Expression::Triple {
                    subject: Subject::V(Variable::new("?X".into())),
                    predicate: Predicate::I("ub:takesCourse".into()),
                    object: Object::I(
                        "<http://www.Department0.University0.edu/GraduateCourse0>".into(),
                    ),
                }),
            ),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y ?Z
/// WHERE {
///   ?X rdf:type ub:GraduateStudent .
///   ?Y rdf:type ub:University .
///   ?Z rdf:type ub:Department .
///   ?X ub:memberOf ?Z .
///   ?Z ub:subOrganizationOf ?Y .
///   ?X ub:undergraduateDegreeFrom ?Y
/// }
/// ```
///
pub fn query2() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V("?X".into()),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:GraduateStudent".into()),
    };

    let t2 = Expression::Triple {
        subject: Subject::V("?Y".into()),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:University".into()),
    };

    let t3 = Expression::Triple {
        subject: Subject::V("?Z".into()),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Department".into()),
    };

    let t4 = Expression::Triple {
        subject: Subject::V("?X".into()),
        predicate: Predicate::I("ub:memberOf".into()),
        object: Object::V("?Z".into()),
    };

    let t5 = Expression::Triple {
        subject: Subject::V("?Z".into()),
        predicate: Predicate::I("ub:subOrganizationOf".into()),
        object: Object::V("?Y".into()),
    };

    let t6 = Expression::Triple {
        subject: Subject::V("?X".into()),
        predicate: Predicate::I("ub:undergraduateDegreeFrom".into()),
        object: Object::V("?Y".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![
                Variable::new("?X".into()),
                Variable::new("?Y".into()),
                Variable::new("?Z".into()),
            ]),
            vec![t1, t2, t3, t4, t5, t6].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:Publication .
///   ?X ub:publicationAuthor <http://www.Department0.University0.edu/AssistantProfessor0>
/// }
/// ```
///
pub fn query3() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Publication".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:publicationAuthor".into()),
        object: Object::I("<http://www.Department0.University0.edu/AssistantProfessor0>".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(Box::new(t1), Box::new(t2)),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y1 ?Y2 ?Y3
/// WHERE {
///   ?X rdf:type ub:Professor .
///   ?X ub:worksFor <http://www.Department0.University0.edu> .
///   ?X ub:name ?Y1 .
///   ?X ub:emailAddress ?Y2 .
///   ?X ub:telephone ?Y3
/// }
/// ```
///
pub fn query4() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Professor".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:worksFor".into()),
        object: Object::I("<http://www.Department0.University0.edu>".into()),
    };
    let t3 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:name".into()),
        object: Object::V(Variable::new("?Y1".into())),
    };
    let t4 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:emailAddress".into()),
        object: Object::V(Variable::new("?Y2".into())),
    };
    let t5 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:telephone".into()),
        object: Object::V(Variable::new("?Y3".into())),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![
                Variable::new("?X".into()),
                Variable::new("?Y1".into()),
                Variable::new("?Y2".into()),
                Variable::new("?Y3".into()),
            ]),
            vec![t1, t2, t3, t4, t5].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:Person .
///   ?X ub:memberOf <http://www.Department0.University0.edu>
/// }
/// ```
///
pub fn query5() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Person".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:memberOf".into()),
        object: Object::I("<http://www.Department0.University0.edu>".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(Box::new(t1), Box::new(t2)),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:Student
/// }
/// ```
///
pub fn query6() -> Query {
    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::Triple {
                subject: Subject::V(Variable::new("?X".into())),
                predicate: Predicate::I("rdf:type".into()),
                object: Object::I("ub:Student".into()),
            },
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y
/// WHERE {
///   ?X rdf:type ub:Student .
///   ?Y rdf:type ub:Course .
///   ?X ub:takesCourse ?Y .
///   <http://www.Department0.University0.edu/AssociateProfessor0> ub:teacherOf ?Y
/// }
/// ```
///
pub fn query7() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Student".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Course".into()),
    };
    let t3 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:takesCourse".into()),
        object: Object::V(Variable::new("?Y".into())),
    };
    let t4 = Expression::Triple {
        subject: Subject::I("<http://www.Department0.University0.edu/AssociateProfessor0>".into()),
        predicate: Predicate::I("ub:teacherOf".into()),
        object: Object::V(Variable::new("?Y".into())),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into()), Variable::new("?Y".into())]),
            vec![t1, t2, t3, t4].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y ?Z
/// WHERE {
///   ?X rdf:type ub:Student .
///   ?Y rdf:type ub:Department .
///   ?X ub:memberOf ?Y .
///   ?Y ub:subOrganizationOf <http://www.University0.edu> .
///   ?X ub:emailAddress ?Z
/// }
/// ```
///
pub fn query8() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Student".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Department".into()),
    };
    let t3 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:memberOf".into()),
        object: Object::V(Variable::new("?Y".into())),
    };
    let t4 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("ub:subOrganizationOf".into()),
        object: Object::I("<http://www.University0.edu>".into()),
    };
    let t5 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:emailAddress".into()),
        object: Object::V(Variable::new("?Z".into())),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![
                Variable::new("?X".into()),
                Variable::new("?Y".into()),
                Variable::new("?Z".into()),
            ]),
            vec![t1, t2, t3, t4, t5].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y ?Z
/// WHERE {
///   ?X rdf:type ub:Student .
///   ?Y rdf:type ub:Faculty .
///   ?Z rdf:type ub:Course .
///   ?X ub:advisor ?Y .
///   ?Y ub:teacherOf ?Z .
///   ?X ub:takesCourse ?Z
/// }
/// ```
///
pub fn query9() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Student".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Faculty".into()),
    };
    let t3 = Expression::Triple {
        subject: Subject::V(Variable::new("?Z".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Course".into()),
    };
    let t4 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:advisor".into()),
        object: Object::V(Variable::new("?Y".into())),
    };
    let t5 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("ub:teacherOf".into()),
        object: Object::V(Variable::new("?Z".into())),
    };
    let t6 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:takesCourse".into()),
        object: Object::V(Variable::new("?Z".into())),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![
                Variable::new("?X".into()),
                Variable::new("?Y".into()),
                Variable::new("?Z".into()),
            ]),
            vec![t1, t2, t3, t4, t5, t6].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:Student .
///   ?X ub:takesCourse <http://www.Department0.University0.edu/GraduateCourse0>
/// }
/// ```
///
pub fn query10() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Student".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:takesCourse".into()),
        object: Object::I("<http://www.Department0.University0.edu/GraduateCourse0>".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(Box::new(t1), Box::new(t2)),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:ResearchGroup .
///   ?X ub:subOrganizationOf <http://www.University0.edu>
/// }
/// ```
///
pub fn query11() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:ResearchGroup".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:subOrganizationOf".into()),
        object: Object::I("<http://www.University0.edu>".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(Box::new(t1), Box::new(t2)),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X ?Y
/// WHERE {
///   ?X rdf:type ub:Chair .
///   ?Y rdf:type ub:Department .
///   ?X ub:worksFor ?Y .
///   ?Y ub:subOrganizationOf <http://www.University0.edu>
/// }
/// ```
///
pub fn query12() -> Query {
    let t1 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Chair".into()),
    };
    let t2 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("rdf:type".into()),
        object: Object::I("ub:Department".into()),
    };
    let t3 = Expression::Triple {
        subject: Subject::V(Variable::new("?X".into())),
        predicate: Predicate::I("ub:worksFor".into()),
        object: Object::V(Variable::new("?Y".into())),
    };
    let t4 = Expression::Triple {
        subject: Subject::V(Variable::new("?Y".into())),
        predicate: Predicate::I("ub:subOrganizationOf".into()),
        object: Object::I("<http://www.University0.edu>".into()),
    };

    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into()), Variable::new("?Y".into())]),
            vec![t1, t2, t3, t4].into_iter().collect(),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:Person .
///   <http://www.University0.edu> ub:hasAlumnus ?X
/// }
/// ```
///
pub fn query13() -> Query {
    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::And(
                Box::new(Expression::Triple {
                    subject: Subject::V(Variable::new("?X".into())),
                    predicate: Predicate::I("rdf:type".into()),
                    object: Object::I("ub:Person".into()),
                }),
                Box::new(Expression::Triple {
                    subject: Subject::I("<http://www.University0.edu>".into()),
                    predicate: Predicate::I("ub:hasAlumnus".into()),
                    object: Object::V(Variable::new("?X".into())),
                }),
            ),
            SolutionModifier::default(),
        ),
    }
}

///
/// ```sparql
/// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
/// PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
/// SELECT ?X
/// WHERE {
///   ?X rdf:type ub:UndergraduateStudent
/// }
/// ```
///
pub fn query14() -> Query {
    Query {
        prologue: prologue(),
        kind: SelectQuery(
            Variables::new(vec![Variable::new("?X".into())]),
            Expression::Triple {
                subject: Subject::V(Variable::new("?X".into())),
                predicate: Predicate::I("rdf:type".into()),
                object: Object::I("ub:UndergraduateStudent".into()),
            },
            SolutionModifier::default(),
        ),
    }
}
