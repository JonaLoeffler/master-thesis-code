use std::collections::{HashMap, HashSet};
use std::option::Option;

use crate::syntax::query::{
    Condition, ConditionVisitor, Expression, ExpressionVisitor, Object, QueryVisitor,
    SolutionModifier, Variable, Variables,
};
use crate::syntax::Iri;
use crate::syntax::{query, Literal};

pub(crate) struct ConditionAnalyzer;

impl ConditionAnalyzer {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
pub struct ConditionInfo {
    map: HashMap<Variable, HashSet<VariableInfo>>,
}

impl ConditionInfo {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, v: Variable, i: VariableInfo) {
        if let Some(infos) = self.map.get_mut(&v) {
            infos.insert(i);
        } else {
            self.map.insert(v, HashSet::from([i]));
        }
    }

    fn invert(self) -> Self {
        let map = self
            .map
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|i| i.invert()).collect()))
            .collect();

        Self { map }
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub(crate) fn union(&self, other: Self) -> Self {
        let mut result = self.clone();

        for (k, v) in other.map.into_iter() {
            for i in v {
                result.insert(k.clone(), i);
            }
        }

        result
    }

    pub(crate) fn get(&self, var: &Variable) -> Option<HashSet<VariableInfo>> {
        self.map.get(var).cloned()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) enum VariableInfo {
    Lt(Literal),
    Gt(Literal),
    Lte(Literal),
    Gte(Literal),

    EqualsLiteral(Literal),
    EqualsIri(Iri),

    NotEqualsLiteral(Literal),
    NotEqualsIri(Iri),

    Bound,
    UnBound,
}

impl VariableInfo {
    fn invert(self) -> Self {
        match self {
            VariableInfo::Lt(v) => VariableInfo::Gte(v),
            VariableInfo::Gt(v) => VariableInfo::Lte(v),
            VariableInfo::Lte(v) => VariableInfo::Gt(v),
            VariableInfo::Gte(v) => VariableInfo::Lt(v),

            VariableInfo::EqualsLiteral(l) => VariableInfo::NotEqualsLiteral(l),
            VariableInfo::EqualsIri(i) => VariableInfo::NotEqualsIri(i),

            VariableInfo::NotEqualsLiteral(l) => VariableInfo::EqualsLiteral(l),
            VariableInfo::NotEqualsIri(i) => VariableInfo::EqualsIri(i),

            VariableInfo::Bound => VariableInfo::UnBound,
            VariableInfo::UnBound => VariableInfo::Bound,
        }
    }
}

impl ConditionVisitor<ConditionInfo> for ConditionAnalyzer {
    fn visit_equals(&mut self, o1: &Object, o2: &Object) -> ConditionInfo {
        let mut result = ConditionInfo::new();

        if let Object::V(v) = o1 {
            if let Object::I(i) = o2 {
                result.insert(v.to_owned(), VariableInfo::EqualsIri(i.to_owned()));
            }
            if let Object::L(l) = o2 {
                result.insert(v.to_owned(), VariableInfo::EqualsLiteral(l.to_owned()));
            }
        }

        if let Object::V(v) = o2 {
            if let Object::I(i) = o1 {
                result.insert(v.to_owned(), VariableInfo::EqualsIri(i.to_owned()));
            }
            if let Object::L(l) = o1 {
                result.insert(v.to_owned(), VariableInfo::EqualsLiteral(l.to_owned()));
            }
        }

        result
    }

    fn visit_gt(&mut self, o1: &Object, o2: &Object) -> ConditionInfo {
        let mut result = ConditionInfo::new();

        if let Object::V(v) = o1 {
            if let Object::L(l) = o2 {
                result.insert(v.to_owned(), VariableInfo::Gt(l.to_owned()));
            }
        }

        if let Object::L(l) = o1 {
            if let Object::V(v) = o2 {
                result.insert(v.to_owned(), VariableInfo::Gt(l.to_owned()));
            }
        }

        result
    }

    fn visit_lt(&mut self, o1: &Object, o2: &Object) -> ConditionInfo {
        let mut result = ConditionInfo::new();

        if let Object::V(v) = o1 {
            if let Object::L(l) = o2 {
                result.insert(v.to_owned(), VariableInfo::Lt(l.to_owned()));
            }
        }

        if let Object::L(l) = o1 {
            if let Object::V(v) = o2 {
                result.insert(v.to_owned(), VariableInfo::Lt(l.to_owned()));
            }
        }

        result
    }

    fn visit_bound(&mut self, v: &Variable) -> ConditionInfo {
        let mut result = ConditionInfo::new();

        result.insert(v.to_owned(), VariableInfo::Bound);

        result
    }

    fn visit_not(&mut self, c: &Condition) -> ConditionInfo {
        ConditionVisitor::visit(self, c).invert()
    }

    fn visit_and(&mut self, c1: &Condition, c2: &Condition) -> ConditionInfo {
        let left = ConditionVisitor::visit(self, c1);
        let right = ConditionVisitor::visit(self, c2);

        left.union(right)
    }

    fn visit_or(&mut self, _c1: &Condition, _c2: &Condition) -> ConditionInfo {
        // let left = ConditionVisitor::visit(self, c1);
        // let right = ConditionVisitor::visit(self, c2);

        // println!("Discarding left info: {:#?}", left);
        // println!("Discarding right info: {:#?}", right);

        ConditionInfo::new()
    }
}

impl<'a> QueryVisitor<'a, ConditionInfo> for ConditionAnalyzer {
    fn visit_select(
        &mut self,
        _: &'a Variables,
        e: &'a Expression,
        m: &'a SolutionModifier,
    ) -> ConditionInfo {
        self.visit_modifier(e, m)
    }

    fn visit_ask(&mut self, e: &'a Expression, m: &'a SolutionModifier) -> ConditionInfo {
        self.visit_modifier(e, m)
    }

    fn visit_modifier(&mut self, e: &'a Expression, _: &'a SolutionModifier) -> ConditionInfo {
        ExpressionVisitor::visit(self, e)
    }
}

impl<'a> ExpressionVisitor<'a, ConditionInfo> for ConditionAnalyzer {
    fn visit_spo(
        &mut self,
        _: &'a query::Subject,
        _: &'a query::Predicate,
        _: &'a Object,
    ) -> ConditionInfo {
        ConditionInfo::new()
    }

    fn visit_and(&mut self, left: &'a Expression, right: &'a Expression) -> ConditionInfo {
        let left = ExpressionVisitor::visit(self, left);
        let right = ExpressionVisitor::visit(self, right);

        left.union(right)
    }

    fn visit_union(&mut self, _: &'a Expression, _: &'a Expression) -> ConditionInfo {
        ConditionInfo::new()
    }

    fn visit_optional(&mut self, left: &'a Expression, _: &'a Expression) -> ConditionInfo {
        ExpressionVisitor::visit(self, left)
    }

    fn visit_filter(&mut self, _: &'a Expression, cond: &'a Condition) -> ConditionInfo {
        ConditionVisitor::visit(self, cond)
    }
}

pub(crate) struct Normalize {}
impl Normalize {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl ConditionVisitor<Condition> for Normalize {
    fn visit_equals(&mut self, o1: &Object, o2: &Object) -> Condition {
        Condition::Equals(o1.clone(), o2.clone())
    }

    fn visit_gt(&mut self, o1: &Object, o2: &Object) -> Condition {
        Condition::GT(o1.clone(), o2.clone())
    }

    fn visit_lt(&mut self, o1: &Object, o2: &Object) -> Condition {
        Condition::LT(o1.clone(), o2.clone())
    }

    fn visit_bound(&mut self, v: &Variable) -> Condition {
        Condition::Bound(v.clone())
    }

    fn visit_not(&mut self, c: &Condition) -> Condition {
        match c {
            Condition::Not(c) => self.visit(c),
            Condition::And(c1, c2) => Condition::Or(
                Box::new(self.visit(&Condition::Not(c1.clone()))),
                Box::new(self.visit(&Condition::Not(c2.clone()))),
            ),
            Condition::Or(c1, c2) => Condition::And(
                Box::new(self.visit(&Condition::Not(c1.clone()))),
                Box::new(self.visit(&Condition::Not(c2.clone()))),
            ),
            _ => Condition::Not(Box::new(self.visit(c))),
        }
    }

    fn visit_and(&mut self, c1: &Condition, c2: &Condition) -> Condition {
        Condition::And(Box::new(self.visit(c1)), Box::new(self.visit(c2)))
    }

    fn visit_or(&mut self, c1: &Condition, c2: &Condition) -> Condition {
        Condition::Or(Box::new(self.visit(c1)), Box::new(self.visit(c2)))
    }
}
