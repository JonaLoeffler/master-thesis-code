use log::{debug, trace};
use tree_sitter::{Node, Tree};

use crate::syntax::{
    query::{
        Condition, Expression, Object, Predicate, SolutionModifier, Subject, Type, Variable,
        Variables,
    },
    Iri,
};
use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

use crate::syntax::query::Query;

trait GetText {
    fn text(&self, bytes: &[u8]) -> String;
}

impl<'a> GetText for tree_sitter::Node<'a> {
    fn text(&self, bytes: &[u8]) -> String {
        self.utf8_text(bytes)
            .expect("should be able to parse node content")
            .to_owned()
    }
}

impl<'a> GetText for tree_sitter::QueryCapture<'a> {
    fn text(&self, bytes: &[u8]) -> String {
        self.node.text(bytes)
    }
}

#[derive(Debug)]
pub enum ParseQueryError {
    EmptySelectClause,
    ParseNodeError(String),
    ParseContentError(Box<dyn Error>),
}

impl Display for ParseQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySelectClause => {
                f.write_str("Select clause must not be empty in this query.")
            }
            Self::ParseNodeError(s) => {
                f.write_str(&format!("Error while parsing where clause: {s}"))
            }
            Self::ParseContentError(e) => f.write_str(&format!("Error while node content: {e}")),
        }
    }
}

impl From<std::num::ParseIntError> for ParseQueryError {
    fn from(e: std::num::ParseIntError) -> Self {
        ParseQueryError::ParseContentError(Box::new(e))
    }
}

impl Error for ParseQueryError {}

impl FromStr for Query {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        log::trace!("Parsing query from str");

        let binding = String::from(s);
        let copy = binding.as_str();
        let bytes = copy.as_bytes();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_sparql::language())?;

        let tree = parser
            .parse(s, Option::None)
            .ok_or("Unable to parse file")?;

        log::trace!("Parsing prologue");

        // Prologue
        let query = tree_sitter::Query::new(
            tree.language(),
            "(prefix_declaration
               (namespace (pn_prefix) @prefix)
               (iri_reference) @iri)",
        )
        .expect("should be able to parse query");

        let mut query_cursor = tree_sitter::QueryCursor::new();
        let declarations = query_cursor
            .matches(&query, tree.root_node(), bytes)
            .map(|result| -> (String, String) {
                let mut captures = result.captures.iter();

                (
                    captures.next().expect("should have capture #1").text(bytes),
                    captures.next().expect("should have capture #2").text(bytes),
                )
            })
            .collect::<HashMap<String, String>>();

        log::trace!("Parsing select");

        // Select
        let query = tree_sitter::Query::new(
            tree.language(),
            "(select_query
               (select_clause
                 bound_variable: (var) @var))",
        )
        .expect("should be able to parse query");

        let mut query_cursor = tree_sitter::QueryCursor::new();
        let variables = query_cursor
            .matches(&query, tree.root_node(), bytes)
            .map(|result| -> Variable {
                let mut captures = result.captures.iter();

                Variable::new(captures.next().expect("should have capture #1").text(bytes))
            })
            .collect::<Variables>();

        if variables.is_empty() {
            return Err(Box::new(ParseQueryError::EmptySelectClause));
        }

        log::trace!("Parsing where");

        // Where
        let query = tree_sitter::Query::new(tree.language(), "(where_clause) @where")
            .expect("should be able to parse query");

        let mut query_cursor = tree_sitter::QueryCursor::new();
        let mut selects = query_cursor.matches(&query, tree.root_node(), bytes);

        let capture = match selects.next() {
            Some(m) => m.captures.iter().next().unwrap(),
            None => {
                return Err(Box::new(ParseQueryError::ParseNodeError(
                    "Failed to unwrap where clause".to_string(),
                )))
            }
        };

        let node = match capture.node.named_child(0) {
            Some(n) => n,
            None => {
                return Err(Box::new(ParseQueryError::ParseNodeError(
                    "Failed to unwrap where clause".to_string(),
                )))
            }
        };

        let expr = match node.kind() {
            "group_graph_pattern" => group_graph_pattern(node, &tree, bytes)?,
            _ => {
                return Err(Box::new(ParseQueryError::ParseNodeError(
                    node.kind().to_string(),
                )))
            }
        };

        log::trace!("Parsing solution modifiers");

        // Solution modifiers
        let query = tree_sitter::Query::new(tree.language(), "(limit_offset_clauses) @limit")
            .expect("should be able to parse query");

        let mut query_cursor = tree_sitter::QueryCursor::new();
        let mut selects = query_cursor.matches(&query, tree.root_node(), bytes);

        let modifier = if let Some(m) = selects.next() {
            let capture = m.captures.iter().next().unwrap();

            match capture.node.kind() {
                "limit_offset_clauses" => limit_offset_clauses(capture.node, &tree, bytes)?,
                _ => {
                    return Err(Box::new(ParseQueryError::ParseNodeError(
                        node.kind().to_string(),
                    )))
                }
            }
        } else {
            SolutionModifier::default()
        };

        log::trace!("Done with parsing query!");

        Ok(Query {
            prologue: declarations,
            kind: Type::SelectQuery(variables, expr, modifier),
        })
    }
}

type ParseResult<T> = Result<T, ParseQueryError>;

fn group_graph_pattern(
    node: Node,
    tree: &Tree,
    bytes: &[u8],
) -> Result<Expression, ParseQueryError> {
    debug!("Parsing group_graph_pattern node");

    let mut expression: Option<Expression> = None;
    let mut condition: Option<Condition> = None;

    for child in node.named_children(&mut tree.walk()) {
        match child.kind() {
            "triples_block" => expression = Some(triples_block(child, tree, bytes)?),
            "filter" => condition = Some(filter(child, tree, bytes)?),
            _ => debug!("Unknown node type {}", child.kind()),
        };
    }

    if let Some(condition) = condition {
        expression = Some(Expression::Filter(
            Box::new(expression.unwrap()),
            Box::new(condition),
        ));
    }

    match expression {
        Some(e) => Ok(e),
        None => Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
    }
}

fn triples_block(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Expression> {
    debug!("Parsing triples_block node");

    let mut triples = vec![];

    for child in node.named_children(&mut tree.walk()) {
        let new_triples = match child.kind() {
            "triples_same_subject" => triples_same_subject(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        triples.push(new_triples);
    }

    Ok(triples.into_iter().collect())
}

fn filter(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Condition> {
    debug!("Parsing filter node");

    if let Some(child) = node.named_children(&mut tree.walk()).next() {
        let condition = match child.kind() {
            "bracketted_expression" => bracketted_expression(child, tree, bytes)?,
            "build_in_function" => build_in_function(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        return Ok(condition);
    }

    Err(ParseQueryError::ParseNodeError(format!("{node:#?}")))
}

fn build_in_function(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Condition> {
    debug!("Parsing builtin function call node");

    for child in node.named_children(&mut tree.walk()) {
        let var = match child.kind() {
            "var" => var(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        if node.text(bytes).to_lowercase().starts_with("bound") {
            return Ok(Condition::Bound(var));
        }
    }

    Err(ParseQueryError::ParseNodeError(format!("{node:#?}")))
}

fn bracketted_expression(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Condition> {
    debug!("Parsing bracketted expression node");

    let mut results: Vec<Condition> = Vec::new();

    for child in node.named_children(&mut tree.walk()) {
        let result = match child.kind() {
            "unary_expression" => unary_expression(child, tree, bytes)?,
            "binary_expression" => binary_expression(child, tree, bytes)?,
            "bracketted_expression" => bracketted_expression(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
        };

        results.push(result);
    }

    if let Some(result) = results.first() {
        return Ok(result.clone());
    }

    Err(ParseQueryError::ParseNodeError(format!("{node:#?}")))
}

fn binary_expression(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Condition> {
    debug!("Parsing binary expression node");

    let operator = node
        .text(bytes)
        .replace(&node.named_child(0).unwrap().text(bytes), "")
        .replace(&node.named_child(1).unwrap().text(bytes), "")
        .trim()
        .to_string();

    if operator == "=" {
        debug!("working on operator {}", operator);

        let left = match node.named_child(0) {
            Some(child) => match child.kind() {
                "var" => Object::V(var(child, tree, bytes)?),
                "rdf_literal" => Object::L(rdf_literal(child, tree, bytes)?.into()),
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("missing{node:#?}"))),
        };

        let right = match node.named_child(1) {
            Some(child) => match child.kind() {
                "var" => Object::V(var(child, tree, bytes)?),
                "rdf_literal" => Object::L(rdf_literal(child, tree, bytes)?.into()),
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        trace!("condition equals {:?}", (&left, &right));

        return Ok(Condition::Equals(left, right));
    }

    if operator == "||" || operator == "&&" {
        debug!("working on operator {}", operator);

        let left = match node.named_child(0) {
            Some(child) => match child.kind() {
                "bracketted_expression" => bracketted_expression(child, tree, bytes)?,
                "build_in_function" => build_in_function(child, tree, bytes)?,
                "unary_expression" => unary_expression(child, tree, bytes)?,
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        let right = match node.named_child(1) {
            Some(child) => match child.kind() {
                "bracketted_expression" => bracketted_expression(child, tree, bytes)?,
                "build_in_function" => build_in_function(child, tree, bytes)?,
                "unary_expression" => unary_expression(child, tree, bytes)?,
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        return match operator.as_str() {
            "||" => Ok(Condition::Or(Box::new(left), Box::new(right))),
            "&&" => Ok(Condition::And(Box::new(left), Box::new(right))),
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };
    }

    if operator == ">" || operator == "<" {
        debug!("working on operator {}", operator);

        let left = match node.named_child(0) {
            Some(child) => match child.kind() {
                "var" => Object::V(var(child, tree, bytes)?),
                "rdf_literal" | "integer" => Object::L(rdf_literal(child, tree, bytes)?.into()),
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        let right = match node.named_child(1) {
            Some(child) => match child.kind() {
                "var" => Object::V(var(child, tree, bytes)?),
                "rdf_literal" | "integer" => Object::L(rdf_literal(child, tree, bytes)?.into()),
                _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
            },
            None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        return match operator.as_str() {
            "<" => Ok(Condition::LT(left, right)),
            ">" => Ok(Condition::GT(left, right)),
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };
    }

    Err(ParseQueryError::ParseNodeError(format!("{node:#?}")))
}

fn unary_expression(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Condition> {
    debug!("Parsing unary expression node");

    match node.child(0).unwrap().kind() {
        "!" => {
            if let Some(child) = node.named_child(0) {
                let condition = match child.kind() {
                    "bracketted_expression" => bracketted_expression(child, tree, bytes)?,
                    _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
                };

                return Ok(Condition::Not(Box::new(condition)));
            }
        }
        _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
    }

    Err(ParseQueryError::ParseNodeError(format!("{node:#?}")))
}

fn triples_same_subject(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Expression> {
    debug!("Parsing triples_same_subject node");

    let mut cursor = tree.walk();
    let mut iter = node.named_children(&mut cursor);

    let subject = match iter.next() {
        Some(child) => match child.kind() {
            "var" => Subject::V(var(child, tree, bytes)?),
            "prefixed_name" => Subject::I(prefixed_name(child, tree, bytes)?),
            "iri_reference" => Subject::I(iri_reference(child, tree, bytes)?),
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        },
        None => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
    };

    let mut triples = vec![];

    for child in iter {
        let properties = match child.kind() {
            "property_list" => property_list(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        for (p, o) in properties {
            triples.push(Expression::Triple(
                Box::new(subject.to_owned()),
                Box::new(p),
                Box::new(o),
            ));
        }
    }

    Ok(triples.into_iter().collect())
}

fn property_list(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Vec<(Predicate, Object)>> {
    debug!("Parsing property_list node");

    let mut properties = vec![];
    for child in node.named_children(&mut tree.walk()) {
        let mut new_properties = match child.kind() {
            "property" => property(child, tree, bytes)?,
            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };

        properties.append(&mut new_properties);
    }

    Ok(properties)
}

fn property(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Vec<(Predicate, Object)>> {
    debug!("Parsing property node");

    let cursor = &mut tree.walk();
    let mut iter = node.named_children(cursor);

    let child = iter.next().unwrap();
    let predicate = match child.kind() {
        "path_element" => Predicate::I(path_element(child, tree, bytes)?),
        "var" => Predicate::V(var(child, tree, bytes)?),
        _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
    };

    let child = iter.next().unwrap();
    let objects = match child.kind() {
        "object_list" => object_list(child, tree, bytes)?,
        _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
    };

    Ok(objects
        .into_iter()
        .map(|o| (predicate.to_owned(), o))
        .collect())
}

fn object_list(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Vec<Object>> {
    debug!("Parsing object_list node");

    node.named_children(&mut tree.walk())
        .map(|child| -> Result<Object, ParseQueryError> {
            match child.kind() {
                "prefixed_name" => Ok(Object::I(prefixed_name(child, tree, bytes)?)),
                "iri_reference" => Ok(Object::I(iri_reference(child, tree, bytes)?)),
                "rdf_literal" => Ok(Object::L(rdf_literal(child, tree, bytes)?.into())),
                "var" => Ok(Object::V(var(child, tree, bytes)?)),
                _ => Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
            }
        })
        .collect()
}

fn rdf_literal(child: Node, _tree: &Tree, bytes: &[u8]) -> ParseResult<String> {
    debug!("Parsing rdf_literal node");

    Ok(child.text(bytes))
}

fn var(node: Node, _tree: &Tree, bytes: &[u8]) -> ParseResult<Variable> {
    debug!("Parsing var node");

    Ok(Variable::new(node.text(bytes)))
}

fn path_element(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<Iri> {
    debug!("Parsing path_element_node");

    if let Some(child) = node.children(&mut tree.walk()).next() {
        return match child.kind() {
            "iri_reference" => iri_reference(child, tree, bytes),
            "prefixed_name" => prefixed_name(child, tree, bytes),
            _ => return Err(ParseQueryError::ParseNodeError(format!("{child:#?}"))),
        };
    }

    Err(ParseQueryError::ParseNodeError(
        "Could not parse path element".to_string(),
    ))
}

fn prefixed_name(node: Node, _tree: &Tree, bytes: &[u8]) -> ParseResult<Iri> {
    debug!("Parsing prefixed_name node");

    let ns = node.child(0).unwrap().child(0).unwrap().text(bytes);
    let local = node.child(1).unwrap().text(bytes);

    Ok(Iri::new(format!("{ns}:{local}")))
}

fn iri_reference(node: Node, _tree: &Tree, bytes: &[u8]) -> ParseResult<Iri> {
    debug!("Parsing prefixed_name node");

    Ok(Iri::IRIREF(node.text(bytes)))
}

fn limit_offset_clauses(node: Node, tree: &Tree, bytes: &[u8]) -> ParseResult<SolutionModifier> {
    debug!("Parsing limit offset clauses {}", node.kind());

    let mut result = SolutionModifier::default();

    for child in node.children(&mut tree.walk()) {
        match child.kind() {
            "limit_clause" => {
                result.with_limit(child.named_child(0).unwrap().text(bytes).parse()?);
            }
            "offset_clause" => {
                result.with_offset(child.named_child(0).unwrap().text(bytes).parse()?);
            }

            _ => return Err(ParseQueryError::ParseNodeError(format!("{node:#?}"))),
        };
    }

    Ok(result)
}
