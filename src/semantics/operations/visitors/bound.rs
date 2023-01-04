use std::collections::HashSet;

use crate::{
    semantics::operations::{
        filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, minus::Minus, offset::Offset,
        projection::Projection, scan::Scan, union::Union, Operation, OperationVisitor,
    },
    syntax::query,
};

pub(crate) struct BoundVars;
impl BoundVars {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<'a, S, J, M, L> OperationVisitor<'a, S, J, M, L, HashSet<query::Variable>> for BoundVars {
    fn visit_scan(&mut self, o: &'a Scan<S, J, M, L>) -> HashSet<query::Variable> {
        let mut result = HashSet::new();

        if let query::Subject::V(v) = &o.subject {
            result.insert(v.to_owned());
        }

        if let query::Predicate::V(v) = &o.predicate {
            result.insert(v.to_owned());
        }

        if let query::Object::V(v) = &o.object {
            result.insert(v.to_owned());
        }

        result
    }

    fn visit_join(&mut self, o: &Join<J, Operation<'a, S, J, M, L>>) -> HashSet<query::Variable> {
        let left = self.visit(&*o.left);
        let right = self.visit(&*o.right);

        left.union(&right).cloned().collect()
    }

    fn visit_projection(
        &mut self,
        o: &Projection<Operation<'a, S, J, M, L>>,
    ) -> HashSet<query::Variable> {
        self.visit(&*o.operation)
            .intersection(&o.vars.iter().cloned().collect())
            .cloned()
            .collect()
    }

    fn visit_union(&mut self, o: &Union<Operation<'a, S, J, M, L>>) -> HashSet<query::Variable> {
        let left = self.visit(&o.left);
        let right = self.visit(&o.right);

        left.intersection(&right).cloned().collect()
    }

    fn visit_filter(
        &mut self,
        o: &'a Filter<Operation<'a, S, J, M, L>>,
    ) -> HashSet<query::Variable> {
        self.visit(&o.operation)
    }

    fn visit_leftjoin(
        &mut self,
        o: &'a LeftJoin<Operation<'a, S, J, M, L>>,
    ) -> HashSet<query::Variable> {
        self.visit(&o.operation)
    }

    fn visit_minus(
        &mut self,
        o: &'a Minus<M, Operation<'a, S, J, M, L>>,
    ) -> HashSet<query::Variable> {
        self.visit(&o.left)
    }

    fn visit_offset(&mut self, o: &Offset<Operation<'a, S, J, M, L>>) -> HashSet<query::Variable> {
        self.visit(&o.operation)
    }

    fn visit_limit(&mut self, o: &Limit<L, Operation<'a, S, J, M, L>>) -> HashSet<query::Variable> {
        self.visit(&o.operation)
    }
}
