use regex::Regex;
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
        // let re_tab = Regex::new(r"^(.*)\t(.*)\t(.*) .$").unwrap();
        let re_quote = Regex::new("^(.*) (.*) (\".*\") .$").unwrap();
        // let re_other = Regex::new(r"^(.*)\s(.*)\s(.*) .$").unwrap();

        let db = s
            .lines()
            .map(|line| -> Triple {
                let subj;
                let pred;
                let mut obj;

                if line.contains('\t') {
                    let mut split = line.split('\t');

                    subj = split.next().unwrap().to_string();
                    pred = split.next().unwrap().to_string();
                    obj = split.next().unwrap().to_string();
                } else if line.contains('\"') {
                    let split = re_quote.captures(line).unwrap();

                    subj = split[1].to_string();
                    pred = split[2].to_string();
                    obj = split[3].to_string();
                } else {
                    let mut split = line.split(' ');

                    subj = split.next().unwrap().to_string();
                    pred = split.next().unwrap().to_string();
                    obj = split.next().unwrap().to_string();
                };

                if obj.ends_with(" .") {
                    obj = obj[..(obj.len() - 2)].to_string();
                }

                let subject = match subj.chars().next() {
                    Some('_') => Subject::B,
                    Some('<') => Subject::I(subj.into()),
                    _ => Subject::B,
                };

                let predicate = Predicate::I(pred.into());

                let object = match obj.chars().next() {
                    Some('_') => Object::B,
                    Some('"') => Object::L(obj.into()),
                    Some('<') => Object::I(obj.into()),
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
        let binding = String::from(s);
        let copy = binding.as_str();
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
            .map(|result| -> (String, String) {
                let mut captures = result.captures.iter();

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
            .flat_map(|result| -> Vec<Triple> {
                let mut triples: Vec<Triple> = vec![];

                let triple = result
                    .captures
                    .iter()
                    .next()
                    .expect("there should be a triple for each match");

                // Match all subjects in the triple
                let query = ts::Query::new(tree.language(), "(subject) @capture")
                    .expect("should be able to parse query");

                let mut query_cursor = ts::QueryCursor::new();
                let node = query_cursor
                    .matches(&query, triple.node, bytes)
                    .next()
                    .expect("only one subject per triple")
                    .captures
                    .get(0)
                    .expect("only one subject per triple");

                let subject = Subject::I(Iri::new(node.expanded(bytes, &prefixes)));

                // Match all properties in the triple
                let query = ts::Query::new(tree.language(), "(property (predicate (_) @capture))")
                    .expect("should be able to parse query");

                for matched in query_cursor.matches(&query, triple.node, bytes) {
                    let capture = matched.captures.get(0).expect("one match in this query");

                    let predicate = Predicate::I(Iri::new(capture.expanded(bytes, &prefixes)));

                    // Match all objects in the triple
                    let query = ts::Query::new(tree.language(), "(object_list (_) @triple)")
                        .expect("should be able to parse query");

                    let mut query_cursor_2 = ts::QueryCursor::new();
                    let next_node = capture.node.parent().expect("").parent().expect("");

                    for next_matched in query_cursor_2.matches(&query, next_node, bytes) {
                        let capture = next_matched
                            .captures
                            .get(0)
                            .expect("one match in this query");

                        let object = match capture.node.kind() {
                            "prefixed_name" => {
                                Object::I(Iri::new(capture.expanded(bytes, &prefixes)))
                            }
                            "iri_reference" => {
                                Object::I(Iri::new(capture.expanded(bytes, &prefixes)))
                            }
                            "rdf_literal" => Object::L(capture.text(bytes).into()),
                            kind => panic!("Unexpected node kind {kind}"),
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
                let mut prefix: String = text.split(':').next().unwrap().into();
                prefix.push(':');

                match prefixes.get(&prefix) {
                    Some(expansion) => format!(
                        "{}{}>",
                        expansion.replace('>', ""),
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
