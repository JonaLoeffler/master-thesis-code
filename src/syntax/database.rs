use itertools::Itertools;
use std::hash::Hash;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::Iri;

#[derive(Debug)]
pub struct Database {
    triples: Vec<Triple>,
    summary: Summary,
}

impl Database {
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
            summary: Summary::new(),
        }
    }

    pub fn add(&mut self, triple: Triple) {
        self.summary.update(&triple);
        self.triples.push(triple);
    }

    pub fn triples(&self) -> &Vec<Triple> {
        &self.triples
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
    }
}

impl FromIterator<Triple> for Database {
    fn from_iter<T: IntoIterator<Item = Triple>>(iter: T) -> Self {
        let mut db = Database::new();

        for i in iter {
            db.add(i);
        }

        db
    }
}

impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for triple in self.triples.iter() {
            f.write_fmt(format_args!(
                "{} {} {} .\n",
                triple.subject, triple.predicate, triple.object,
            ))?;
        }

        return Ok(());
    }
}

#[derive(Debug, Clone)]
pub struct Triple {
    pub subject: Subject,
    pub predicate: Predicate,
    pub object: Object,
}

impl Triple {
    pub fn new(subject: Subject, predicate: Predicate, object: Object) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Subject {
    B,
    I(Iri),
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::B => Ok(f.write_str("()")?),
            Subject::I(u) => u.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Predicate {
    I(Iri),
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::I(u) => u.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Object {
    B,
    L(String),
    I(Iri),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Object::B => f.write_str("()"),
            Object::L(l) => f.write_str(&format!("{}", l)),
            Object::I(u) => f.write_str(&format!("{}", u)),
        }
    }
}

#[derive(Debug)]
pub struct Summary {
    /// Total number of triples
    t: u32,

    /// Distinct subjects
    r: HashSet<Subject>,

    /// Number of triples per predicate
    t_p: HashMap<Predicate, u32>,

    /// Number of triples per predicate and object
    pub o_c: HashMap<Predicate, HashMap<Object, u32>>,
}

impl Summary {
    pub fn t(&self) -> f32 {
        let tmp: u32 = self.t.try_into().unwrap();
        tmp as f32
    }
    pub fn r(&self) -> f32 {
        let tmp: u32 = self.r.len().try_into().unwrap();
        tmp as f32
    }
    pub fn t_p(&self, p: &Predicate) -> f32 {
        if let Some(count) = self.t_p.get(p) {
            *count as f32
        } else {
            0.0
        }
    }
    pub fn o_c(&self, p: &Predicate, o: &Object) -> f32 {
        let result = if let Some(predmap) = self.o_c.get(p) {
            if let Some(count) = predmap.get(o) {
                *count as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        result
    }

    pub fn new() -> Summary {
        Summary {
            t: 0,
            r: HashSet::new(),
            t_p: HashMap::new(),
            o_c: HashMap::new(),
        }
    }

    fn update(&mut self, triple: &Triple) -> () {
        // update T
        self.t += 1;

        // Update R
        self.r.insert(triple.subject.to_owned());

        // Update T_P
        if let Some(count) = self.t_p.get_mut(&triple.predicate) {
            *count += 1;
        } else {
            self.t_p.insert(triple.predicate.to_owned(), 1);
        }

        // Update O_c
        if let Some(predmap) = self.o_c.get_mut(&triple.predicate) {
            if let Some(count) = predmap.get_mut(&triple.object) {
                *count += 1;
            } else {
                predmap.insert(triple.object.to_owned(), 1);
            }
        } else {
            self.o_c.insert(
                triple.predicate.to_owned(),
                HashMap::from([(triple.object.to_owned(), 1)]),
            );
        }
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Database Statistics:\n")?;
        f.write_str(&format!("T: {}\n", self.t()))?;
        f.write_str(&format!("R: {}\n", self.r()))?;
        f.write_str(&format!("T_P: {}\n", self.t_p.len()))?;
        f.write_str(&format!("O_c: {} ", self.o_c.len()))?;

        let iter = self.o_c.iter().map(|(_, v)| v.len().to_string());
        let res = Itertools::intersperse(iter, ", ".to_string());
        f.write_str(&format!("({})", res.collect::<String>()))?;
        f.write_str("\n")?;

        Ok(())
    }
}
