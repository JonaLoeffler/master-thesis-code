use std::{collections::HashMap, error::Error, fmt::Display};

use tree_sitter as ts;

use crate::syntax::{
    database::{Database, Object, Predicate, Subject, Triple},
    Iri,
};

#[derive(Debug)]
pub enum ParseDatabaseError {}

impl Error for ParseDatabaseError {}

impl Display for ParseDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cannot parse Database")
    }
}

impl Database {
    pub fn from_ntriples_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let db = s
            .lines()
            .map(|line| -> Triple {
                let mut split = line.split(" ");

                let subject = split.next().expect("Should have subject");
                let subject = match subject.chars().next() {
                    Some('<') => Subject::I(Iri::new(subject.to_string())),
                    _ => Subject::B,
                };

                let predicate = Predicate::I(Iri::new(
                    split.next().expect("Should have predicate").to_string(),
                ));

                let object = split.next().expect("Should have object");
                let object = match object.chars().next() {
                    Some('<') => Object::I(Iri::new(object.to_string())),
                    Some('"') => Object::L(object.to_string()),
                    _ => Object::B,
                };

                Triple {
                    subject,
                    predicate,
                    object,
                }
            })
            .collect::<Database>();

        Ok(db)
    }

    pub fn from_turtle_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let copy = s.clone();
        let bytes = copy.as_bytes();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_turtle::language())?;

        let tree = parser
            .parse(s, Option::None)
            .ok_or("Unable to parse file")?;

        // Match all prefixes
        let query = ts::Query::new(
            tree.language(),
            "(statement (directive (prefix_id (namespace) @namespace (iri_reference) @iri)))",
        )
        .expect("should be able to parse query");

        let mut query_cursor = ts::QueryCursor::new();
        let prefixes = query_cursor
            .matches(&query, tree.root_node(), bytes)
            .into_iter()
            .map(|result| -> (String, String) {
                let mut captures = result.captures.into_iter();

                (
                    captures.next().expect("should have namespace").text(bytes),
                    captures.next().expect("should have iri").text(bytes),
                )
            })
            .collect::<HashMap<String, String>>();

        // Match all triples
        let query = ts::Query::new(tree.language(), "(statement (triples) @triple)")
            .expect("should be able to parse query");

        let mut query_cursor = ts::QueryCursor::new();
        let db = query_cursor
            .matches(&query, tree.root_node(), bytes)
            .into_iter()
            .flat_map(|result| -> Vec<Triple> {
                let mut triples: Vec<Triple> = vec![];

                let triple = result
                    .captures
                    .into_iter()
                    .next()
                    .expect("there should be a triple for each match");

                // Match all subjects in the triple
                let query = ts::Query::new(tree.language(), "(subject) @capture")
                    .expect("should be able to parse query");

                let mut query_cursor = ts::QueryCursor::new();
                let node = query_cursor
                    .matches(&query, triple.node, bytes)
                    .into_iter()
                    .nth(0)
                    .expect("only one subject per triple")
                    .captures
                    .into_iter()
                    .nth(0)
                    .expect("only one subject per triple");

                let subject = Subject::I(Iri::new(node.expanded(bytes, &prefixes)));

                // Match all properties in the triple
                let query = ts::Query::new(tree.language(), "(property (predicate (_) @capture))")
                    .expect("should be able to parse query");

                for matched in query_cursor.matches(&query, triple.node, bytes) {
                    let capture = matched
                        .captures
                        .into_iter()
                        .nth(0)
                        .expect("one match in this query");

                    let predicate = Predicate::I(Iri::new(capture.expanded(bytes, &prefixes)));

                    // Match all objects in the triple
                    let query = ts::Query::new(tree.language(), "(object_list (_) @triple)")
                        .expect("should be able to parse query");

                    let mut query_cursor_2 = ts::QueryCursor::new();
                    let next_node = capture.node.parent().expect("").parent().expect("");

                    for next_matched in query_cursor_2.matches(&query, next_node, bytes) {
                        let capture = next_matched
                            .captures
                            .into_iter()
                            .nth(0)
                            .expect("one match in this query");

                        let object = match capture.node.kind() {
                            "prefixed_name" => {
                                Object::I(Iri::new(capture.expanded(bytes, &prefixes)))
                            }
                            "iri_reference" => {
                                Object::I(Iri::new(capture.expanded(bytes, &prefixes)))
                            }
                            "rdf_literal" => Object::L(capture.text(bytes)),
                            kind => panic!("Unexpected node kind {}", kind),
                        };

                        triples.push(Triple {
                            subject: subject.clone(),
                            predicate: predicate.clone(),
                            object,
                        });
                    }
                }

                triples
            })
            .collect::<Database>();

        Ok(db)
    }
}

trait GetText {
    fn text(&self, bytes: &[u8]) -> String;
    fn expanded(&self, bytes: &[u8], prefixes: &HashMap<String, String>) -> String;
}

impl<'a> GetText for ts::Node<'a> {
    fn text(&self, bytes: &[u8]) -> String {
        self.utf8_text(bytes)
            .expect("should be able to parse node content")
            .to_owned()
    }

    fn expanded(&self, bytes: &[u8], prefixes: &HashMap<String, String>) -> String {
        let text = self.text(bytes);

        match self.kind() {
            "prefixed_name" => {
                let mut prefix: String = text.split(":").nth(0).unwrap().into();
                prefix.push_str(":");

                match prefixes.get(&prefix) {
                    Some(expansion) => format!(
                        "{}{}>",
                        expansion.replace(">", ""),
                        text.replace(&prefix, "")
                    ),
                    _ => text,
                }
            }
            _ => text,
        }
    }
}

impl<'a> GetText for ts::QueryCapture<'a> {
    fn text(&self, bytes: &[u8]) -> String {
        self.node.text(bytes)
    }

    fn expanded(&self, bytes: &[u8], prefixes: &HashMap<String, String>) -> String {
        self.node.expanded(bytes, prefixes)
    }
}
