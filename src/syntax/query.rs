use itertools::Itertools;

use crate::syntax::Iri;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::{Expand, ExpandError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Query {
    pub(crate) prologue: HashMap<String, String>,
    pub(crate) kind: Type,
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (pn_prefix, iri_reference) in self.prologue.iter() {
            f.write_str(&format!("PREFIX {}: {}\n", pn_prefix, iri_reference))?;
        }

        match &self.kind {
            Type::SelectQuery(vars, expr, modifier) => {
                f.write_str("SELECT")?;

                for var in vars.iter() {
                    f.write_str(&format!(" {}", var.name))?;
                }

                f.write_str("\nWHERE {\n")?;

                expr.fmt(f)?;

                f.write_str("\n}\n")?;

                if let Some(limit) = modifier.limit {
                    f.write_str(&format!("LIMIT {limit}\n"))?;
                }

                if let Some(offset) = modifier.offset {
                    f.write_str(&format!("OFFSET {offset}\n"))?;
                }
            }
            Type::AskQuery(expr, modifier) => {
                f.write_str("ASK")?;

                f.write_str("\nWHERE {\n")?;

                expr.fmt(f)?;

                f.write_str("\n}\n")?;

                if let Some(limit) = modifier.limit {
                    f.write_str(&format!("LIMIT {limit}\n"))?;
                }

                if let Some(offset) = modifier.offset {
                    f.write_str(&format!("OFFSET {offset}\n"))?;
                }
            }
        }

        Ok(())
    }
}

impl Query {
    pub fn expand(self) -> Result<Query, ExpandError> {
        match self.kind {
            Type::SelectQuery(v, e, m) => {
                let e = e.expand(&self.prologue)?;

                Ok(Query {
                    prologue: self.prologue,
                    kind: Type::SelectQuery(v, e, m),
                })
            }
            Type::AskQuery(e, m) => {
                let e = e.expand(&self.prologue)?;

                Ok(Query {
                    prologue: self.prologue,
                    kind: Type::AskQuery(e, m),
                })
            }
        }
    }
}

impl Expand for Expression {
    type Expandable = Expression;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Expression, ExpandError> {
        match self {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => Ok(Expression::Triple {
                subject: subject.expand(&prefixes)?,
                predicate: predicate.expand(&prefixes)?,
                object: object.expand(&prefixes)?,
            }),
            Expression::And(e1, e2) => Ok(Expression::And(
                Box::new(e1.expand(&prefixes)?),
                Box::new(e2.expand(&prefixes)?),
            )),
            Expression::Union(e1, e2) => Ok(Expression::Union(
                Box::new(e1.expand(&prefixes)?),
                Box::new(e2.expand(&prefixes)?),
            )),
            Expression::Optional(e1, e2) => Ok(Expression::Optional(
                Box::new(e1.expand(&prefixes)?),
                Box::new(e2.expand(&prefixes)?),
            )),
            Expression::Filter(e, r) => Ok(Expression::Filter(Box::new(e.expand(&prefixes)?), r)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Type {
    SelectQuery(Variables, Expression, SolutionModifier),
    AskQuery(Expression, SolutionModifier),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct SolutionModifier {
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

impl SolutionModifier {
    pub(crate) fn with_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    pub(crate) fn with_offset(&mut self, offset: usize) {
        self.offset = Some(offset);
    }
}

impl Default for SolutionModifier {
    fn default() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct Variable {
    pub(crate) name: String,
    position: Option<usize>,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            position: None,
        }
    }

    pub fn set_pos(self, position: usize) -> Self {
        Self {
            name: self.name,
            position: Some(position),
        }
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.position.is_some() && other.position.is_some() {
            self.position.unwrap().partial_cmp(&other.position.unwrap())
        } else {
            self.name.partial_cmp(&other.name)
        }
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.position.is_some() && other.position.is_some() {
            self.position.unwrap().cmp(&other.position.unwrap())
        } else {
            self.name.cmp(&other.name)
        }
    }
}

impl From<String> for Variable {
    fn from(s: String) -> Self {
        Variable::new(s)
    }
}

impl From<&str> for Variable {
    fn from(s: &str) -> Self {
        Variable::new(s.to_string())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct Variables(Vec<Variable>);

impl Variables {
    pub(crate) fn new(vars: Vec<Variable>) -> Self {
        Self(vars)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<Variable> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<Variable> for Variables {
    fn from_iter<T: IntoIterator<Item = Variable>>(iter: T) -> Self {
        let mut vars = Vec::new();

        for var in iter {
            vars.push(var);
        }

        Variables::new(vars)
    }
}

impl From<HashSet<Variable>> for Variables {
    fn from(set: HashSet<Variable>) -> Self {
        set.into_iter().collect()
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vars = self.0.iter().map(|v| v.name.to_owned());
        let res = Itertools::intersperse(vars, ", ".to_string());

        f.write_str(&format!("[{}]", res.collect::<String>()))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Expression {
    Triple {
        subject: Subject,
        predicate: Predicate,
        object: Object,
    },
    And(Box<Expression>, Box<Expression>),
    Union(Box<Expression>, Box<Expression>),
    Optional(Box<Expression>, Box<Expression>),
    Filter(Box<Expression>, Box<Condition>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => {
                f.write_str("  {")?;
                subject.fmt(f)?;
                f.write_str(" ")?;
                predicate.fmt(f)?;
                f.write_str(" ")?;
                object.fmt(f)?;
                f.write_str("}")?;
                Ok(())
            }
            Expression::And(e1, e2) => {
                e1.fmt(f)?;
                f.write_str(" . \n")?;
                e2.fmt(f)?;
                Ok(())
            }
            Expression::Union(e1, e2) => {
                e1.fmt(f)?;
                f.write_str("    UNION\n")?;
                e2.fmt(f)?;
                Ok(())
            }
            Expression::Optional(e1, e2) => {
                e1.fmt(f)?;
                f.write_str("    OPTIONAL")?;
                e2.fmt(f)?;
                Ok(())
            }
            Expression::Filter(e1, r) => {
                e1.fmt(f)?;
                f.write_str("    FILTER (")?;
                r.fmt(f)?;
                f.write_str(")\n")?;
                Ok(())
            }
        }
    }
}

impl FromIterator<Expression> for Expression {
    fn from_iter<T: IntoIterator<Item = Expression>>(iter: T) -> Self {
        let mut root: Option<Expression> = None;

        for e in iter {
            match root {
                Some(prev) => root = Some(Expression::And(Box::new(prev), Box::new(e))),
                None => root = Some(e),
            }
        }

        root.unwrap()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Subject {
    I(Iri),
    V(Variable),
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Subject::I(u) => f.write_str(&format!("{}", u)),
            Subject::V(v) => f.write_str(&format!("{}", v.name)),
        }
    }
}

impl Expand for Subject {
    type Expandable = Subject;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Subject, ExpandError> {
        match self {
            Subject::I(s) => Ok(Subject::I(s.expand(&prefixes)?)),
            Subject::V(v) => Ok(Subject::V(v)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Predicate {
    I(Iri),
    V(Variable),
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Predicate::I(u) => f.write_str(&format!("{}", u)),
            Predicate::V(v) => f.write_str(&format!("{}", v.name)),
        }
    }
}

impl Expand for Predicate {
    type Expandable = Predicate;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Predicate, ExpandError> {
        match self {
            Predicate::I(s) => Ok(Predicate::I(s.expand(prefixes)?)),
            Predicate::V(v) => Ok(Predicate::V(v)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Object {
    L(String),
    I(Iri),
    V(Variable),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Object::L(l) => f.write_str(&format!("{}", l)),
            Object::I(u) => f.write_str(&format!("{}", u)),
            Object::V(v) => f.write_str(&format!("{}", v.name)),
        }
    }
}

impl Expand for Object {
    type Expandable = Object;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Object, ExpandError> {
        match self {
            Object::I(s) => Ok(Object::I(s.expand(prefixes)?)),
            Object::V(v) => Ok(Object::V(v)),
            Object::L(l) => Ok(Object::L(l)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Condition {
    Equals(Object, Object),
    Bound(Variable),
    Not(Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Condition::Equals(o1, o2) => f.write_str(&format!("{} = {}", o1, o2)),
            Condition::Bound(v) => f.write_str(&format!("{}", v.name)),
            Condition::Not(c) => f.write_str(&format!("¬({})", c)),
            Condition::And(c1, c2) => f.write_str(&format!("({}) ∧ ({})", c1, c2)),
            Condition::Or(c1, c2) => f.write_str(&format!("({}) ∨ ({})", c1, c2)),
        }
    }
}

pub(crate) trait QueryVisitor<'a, T> {
    fn visit(&mut self, o: &'a Query) -> T {
        match &o.kind {
            Type::SelectQuery(v, e, m) => self.visit_select(v, e, m),
            Type::AskQuery(e, m) => self.visit_ask(e, m),
        }
    }

    fn visit_select(
        &mut self,
        vars: &'a Variables,
        expr: &'a Expression,
        modifier: &'a SolutionModifier,
    ) -> T;

    fn visit_ask(&mut self, expr: &'a Expression, modifier: &'a SolutionModifier) -> T;

    fn visit_expr(&mut self, o: &'a Expression) -> T {
        match o {
            Expression::Triple {
                subject,
                predicate,
                object,
            } => self.visit_spo(subject, predicate, object),
            Expression::And(left, right) => self.visit_and(left, right),
            Expression::Union(left, right) => self.visit_union(left, right),
            Expression::Optional(left, right) => self.visit_optional(left, right),
            Expression::Filter(expr, cond) => self.visit_filter(expr, cond),
        }
    }

    fn visit_spo(
        &mut self,
        subject: &'a Subject,
        predicate: &'a Predicate,
        object: &'a Object,
    ) -> T;
    fn visit_and(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_union(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_optional(&mut self, left: &'a Expression, right: &'a Expression) -> T;
    fn visit_filter(&mut self, expr: &'a Expression, cond: &'a Condition) -> T;
    fn visit_modifier(&mut self, expr: &'a Expression, modifier: &'a SolutionModifier) -> T;
}
