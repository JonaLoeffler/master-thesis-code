use std::{collections::HashMap, error::Error, fmt::Display};

use super::{
    query::{
        Condition, Expression, ExpressionVisitor, Object, Predicate, Query, QueryVisitor,
        SolutionModifier, Subject, Type, Variables,
    },
    Iri, PrefixedName,
};

pub(crate) struct Expand {
    prologue: HashMap<String, String>,
}

impl Expand {
    pub(crate) fn new(prologue: HashMap<String, String>) -> Self {
        Self { prologue }
    }
}

#[derive(Debug)]
pub enum ExpandError {
    PrefixNotFound(String),
}

impl Error for ExpandError {}

impl Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::PrefixNotFound(s) => f.write_str(&format!("Prefix \'{s}\' not found")),
        }
    }
}

type ExpandResult<T> = Result<T, ExpandError>;

impl<'a> QueryVisitor<'a, ExpandResult<Query>> for Expand {
    fn visit_select(
        &mut self,
        v: &'a Variables,
        e: &'a Expression,
        m: &'a SolutionModifier,
    ) -> ExpandResult<Query> {
        Ok(Query {
            prologue: self.prologue.clone(),
            kind: Type::SelectQuery(v.clone(), ExpressionVisitor::visit(self, e)?, m.clone()),
        })
    }

    fn visit_ask(&mut self, e: &'a Expression, m: &'a SolutionModifier) -> ExpandResult<Query> {
        Ok(Query {
            prologue: self.prologue.clone(),
            kind: Type::AskQuery(ExpressionVisitor::visit(self, e)?, m.clone()),
        })
    }

    fn visit_modifier(
        &mut self,
        _e: &'a Expression,
        _m: &'a SolutionModifier,
    ) -> ExpandResult<Query> {
        todo!()
    }
}

impl<'a> ExpressionVisitor<'a, ExpandResult<Expression>> for Expand {
    fn visit_spo(
        &mut self,
        s: &'a Subject,
        p: &'a Predicate,
        o: &'a Object,
    ) -> ExpandResult<Expression> {
        Ok(Expression::Triple(
            Box::new({
                match s {
                    Subject::I(i) => Subject::I(i.clone().expand(&self.prologue)?),
                    Subject::V(v) => Subject::V(v.clone()),
                }
            }),
            Box::new({
                match p {
                    Predicate::I(i) => Predicate::I(i.clone().expand(&self.prologue)?),
                    Predicate::V(v) => Predicate::V(v.clone()),
                }
            }),
            Box::new(match o {
                Object::L(l) => Object::L(l.clone()),
                Object::I(i) => Object::I(i.clone().expand(&self.prologue)?),
                Object::V(v) => Object::V(v.clone()),
            }),
        ))
    }

    fn visit_and(&mut self, l: &'a Expression, r: &'a Expression) -> ExpandResult<Expression> {
        Ok(Expression::And(
            Box::new(ExpressionVisitor::visit(self, l)?),
            Box::new(ExpressionVisitor::visit(self, r)?),
        ))
    }

    fn visit_union(&mut self, l: &'a Expression, r: &'a Expression) -> ExpandResult<Expression> {
        Ok(Expression::Union(
            Box::new(ExpressionVisitor::visit(self, l)?),
            Box::new(ExpressionVisitor::visit(self, r)?),
        ))
    }

    fn visit_optional(&mut self, l: &'a Expression, r: &'a Expression) -> ExpandResult<Expression> {
        Ok(Expression::Optional(
            Box::new(ExpressionVisitor::visit(self, l)?),
            Box::new(ExpressionVisitor::visit(self, r)?),
        ))
    }

    fn visit_filter(&mut self, e: &'a Expression, c: &'a Condition) -> ExpandResult<Expression> {
        Ok(Expression::Filter(
            Box::new(ExpressionVisitor::visit(self, e)?),
            Box::new(c.clone()),
        ))
    }
}

impl Iri {
    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Iri, ExpandError> {
        match self {
            Self::IRIREF(s) => Ok(Self::IRIREF(s)),
            Self::PrefixedName(name) => {
                if name.ns == "_" {
                    // The item is a blank node and cannot be expanded
                    return Ok(Self::PrefixedName(name));
                }

                let expanded = if let Some(expansion) = prefixes.get(&name.ns) {
                    Some(format!("{}{}>", expansion.replace('>', ""), name.local))
                } else {
                    return Err(ExpandError::PrefixNotFound(name.ns));
                };

                Ok(Self::PrefixedName(PrefixedName {
                    ns: name.ns,
                    local: name.local,
                    expanded,
                }))
            }
        }
    }
}
