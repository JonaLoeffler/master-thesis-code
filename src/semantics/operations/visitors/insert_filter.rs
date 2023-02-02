use super::{
    bound::BoundVars,
    condition::{ConditionInfo, VariableInfo},
};
use crate::{
    semantics::operations::{
        filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, minus::Minus, offset::Offset,
        projection::Projection, scan::Scan, union::Union, Operation, OperationVisitor,
    },
    syntax::{query::Condition, query::Object},
};

pub(crate) struct FilterInserter {
    info: ConditionInfo,
}

impl FilterInserter {
    pub(crate) fn new(info: ConditionInfo) -> Self {
        Self { info }
    }
}

impl<'a> OperationVisitor<'a, Operation<'a>> for FilterInserter {
    fn visit_scan(&mut self, o: &'a Scan) -> Operation<'a> {
        let mut condition = None;

        for v in BoundVars::new().visit_scan(o) {
            if let Some(infos) = self.info.get(&v) {
                for info in infos.iter() {
                    let next = match info {
                        VariableInfo::Lt(l) => {
                            Condition::LT(Object::V(v.to_owned()), Object::L(l.to_owned()))
                        }
                        VariableInfo::Gt(l) => {
                            Condition::GT(Object::V(v.to_owned()), Object::L(l.to_owned()))
                        }
                        VariableInfo::Lte(l) => Condition::Not(Box::new(Condition::GT(
                            Object::V(v.to_owned()),
                            Object::L(l.to_owned()),
                        ))),
                        VariableInfo::Gte(l) => Condition::Not(Box::new(Condition::LT(
                            Object::V(v.to_owned()),
                            Object::L(l.to_owned()),
                        ))),
                        VariableInfo::EqualsIri(i) => {
                            Condition::Equals(Object::V(v.to_owned()), Object::I(i.to_owned()))
                        }
                        VariableInfo::EqualsLiteral(l) => {
                            Condition::Equals(Object::V(v.to_owned()), Object::L(l.to_owned()))
                        }
                        VariableInfo::NotEqualsLiteral(l) => Condition::Not(Box::new(
                            Condition::Equals(Object::V(v.to_owned()), Object::L(l.to_owned())),
                        )),
                        VariableInfo::NotEqualsIri(i) => Condition::Not(Box::new(
                            Condition::Equals(Object::V(v.to_owned()), Object::I(i.to_owned())),
                        )),
                        VariableInfo::Bound => Condition::Bound(v.to_owned()),
                        VariableInfo::UnBound => {
                            Condition::Not(Box::new(Condition::Bound(v.to_owned())))
                        }
                    };

                    condition = if let Some(c) = condition {
                        Some(Condition::And(Box::new(c), Box::new(next)))
                    } else {
                        Some(next)
                    }
                }
            }
        }

        match condition {
            Some(c) => Operation::Filter(Filter::new(Operation::Scan(o.clone()), c)),
            None => Operation::Scan(o.clone()),
        }
    }

    fn visit_join(&mut self, o: &'a Join<Operation<'a>>) -> Operation<'a> {
        Operation::Join(Join::new(self.visit(&o.left), self.visit(&o.right)))
    }

    fn visit_projection(&mut self, o: &'a Projection<Operation<'a>>) -> Operation<'a> {
        Operation::Projection(Projection::new(self.visit(&o.operation), o.vars.clone()))
    }

    fn visit_union(&mut self, o: &'a Union<Operation<'a>>) -> Operation<'a> {
        Operation::Union(Union::new(self.visit(&o.left), self.visit(&o.right)))
    }

    fn visit_filter(&mut self, o: &'a Filter<Operation<'a>>) -> Operation<'a> {
        Operation::Filter(Filter::new(self.visit(&o.operation), *o.condition.clone()))
    }

    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a>>) -> Operation<'a> {
        Operation::LeftJoin(LeftJoin::new(self.visit(&o.left), self.visit(&o.right)))
    }

    fn visit_minus(&mut self, o: &'a Minus<Operation<'a>>) -> Operation<'a> {
        Operation::Minus(Minus::new(self.visit(&o.left), self.visit(&o.right)))
    }

    fn visit_offset(&mut self, o: &'a Offset<Operation<'a>>) -> Operation<'a> {
        Operation::Offset(Offset::new(self.visit(&o.operation), o.offset))
    }

    fn visit_limit(&mut self, o: &'a Limit<Operation<'a>>) -> Operation<'a> {
        Operation::Limit(Limit::new(self.visit(&o.operation), o.limit))
    }
}
