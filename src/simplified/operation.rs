use crate::syntax::database;

use super::{
    query_ast::{Object, Predicate, Subject, Variable},
    Mapping, MappingSet,
};

pub enum Operation<'a> {
    Scan(Scan<'a>),
    Join(Join<'a>),
    // Other operations omitted
}

pub struct Scan<'a> {
    db: &'a database::Database,
    subject: Subject,
    predicate: Predicate,
    object: Object,
    // additional state for iterator implementation omitted
}

pub struct Join<'a> {
    left: Box<Operation<'a>>,
    right: Box<Operation<'a>>,
    join_vars: Vec<Variable>,
    // additional state for iterator implementation omitted
}

impl<'a> Iterator for Scan<'a> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> Iterator for Join<'a> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
