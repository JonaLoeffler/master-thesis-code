#[cfg(test)]
mod lubm {
    use crate::{examples as ex, syntax::query::Query};

    #[test]
    fn query1() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE
{
  ?X rdf:type ub:GraduateStudent .
  ?X ub:takesCourse <http://www.Department0.University0.edu/GraduateCourse0>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query1()
        );
    }

    #[test]
    fn query2() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y ?Z
WHERE
{?X rdf:type ub:GraduateStudent .
  ?Y rdf:type ub:University .
  ?Z rdf:type ub:Department .
  ?X ub:memberOf ?Z .
  ?Z ub:subOrganizationOf ?Y .
  ?X ub:undergraduateDegreeFrom ?Y
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query2()
        );
    }

    #[test]
    fn query3() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:Publication .
  ?X ub:publicationAuthor <http://www.Department0.University0.edu/AssistantProfessor0>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query3()
        );
    }

    #[test]
    fn query4() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y1 ?Y2 ?Y3
WHERE {
  ?X rdf:type ub:Professor .
  ?X ub:worksFor <http://www.Department0.University0.edu> .
  ?X ub:name ?Y1 .
  ?X ub:emailAddress ?Y2 .
  ?X ub:telephone ?Y3
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query4()
        );
    }

    #[test]
    fn query5() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:Person .
  ?X ub:memberOf <http://www.Department0.University0.edu>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query5()
        );
    }

    #[test]
    fn query6() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:Student
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query6()
        );
    }

    #[test]
    fn query7() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y
WHERE {
  ?X rdf:type ub:Student .
  ?Y rdf:type ub:Course .
  ?X ub:takesCourse ?Y .
  <http://www.Department0.University0.edu/AssociateProfessor0> ub:teacherOf ?Y
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query7()
        );
    }

    #[test]
    fn query8() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y ?Z
WHERE {
  ?X rdf:type ub:Student .
  ?Y rdf:type ub:Department .
  ?X ub:memberOf ?Y .
  ?Y ub:subOrganizationOf <http://www.University0.edu> .
  ?X ub:emailAddress ?Z
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query8()
        );
    }

    #[test]
    fn query9() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y ?Z
WHERE {
  ?X rdf:type ub:Student .
  ?Y rdf:type ub:Faculty .
  ?Z rdf:type ub:Course .
  ?X ub:advisor ?Y .
  ?Y ub:teacherOf ?Z .
  ?X ub:takesCourse ?Z
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query9()
        );
    }

    #[test]
    fn query10() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:Student .
  ?X ub:takesCourse <http://www.Department0.University0.edu/GraduateCourse0>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query10()
        );
    }

    #[test]
    fn query11() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:ResearchGroup .
  ?X ub:subOrganizationOf <http://www.University0.edu>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query11()
        );
    }

    #[test]
    fn query12() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X ?Y
WHERE {
  ?X rdf:type ub:Chair .
  ?Y rdf:type ub:Department .
  ?X ub:worksFor ?Y .
  ?Y ub:subOrganizationOf <http://www.University0.edu>
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query12()
        );
    }

    #[test]
    fn query13() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {
  ?X rdf:type ub:Person .
  <http://www.University0.edu> ub:hasAlumnus ?X
}"
            .parse::<Query>()
            .unwrap(),
            ex::lubm::query13()
        );
    }

    #[test]
    fn query14() {
        assert_eq!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>
SELECT ?X
WHERE {?X rdf:type ub:UndergraduateStudent}"
                .parse::<Query>()
                .unwrap(),
            ex::lubm::query14()
        );
    }
}
