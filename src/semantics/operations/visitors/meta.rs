use std::ops::Add;

use crate::semantics::{
    operations::{
        filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, minus::Minus, offset::Offset,
        projection::Projection, scan::Scan, union::Union, Operation, OperationVisitor,
    },
    results::OperationMeta,
};

pub(crate) struct Meta {}

impl Meta {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Add<Self> for OperationMeta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            joins: self.joins + rhs.joins,
            scans: self.scans + rhs.scans,
            disjunct_joins: self.disjunct_joins + rhs.disjunct_joins,
        }
    }
}

impl<'a, S, J, M, L> OperationVisitor<'a, S, J, M, L, OperationMeta> for Meta {
    fn visit_scan(&mut self, _o: &'a Scan<S, J, M, L>) -> OperationMeta {
        OperationMeta {
            joins: 0,
            scans: 1,
            disjunct_joins: 0,
        }
    }

    fn visit_join(&mut self, o: &'a Join<J, Operation<'a, S, J, M, L>>) -> OperationMeta {
        OperationMeta {
            joins: 1,
            scans: 0,
            disjunct_joins: if o.join_vars().is_empty() { 1 } else { 0 },
        } + self.visit(&o.left)
            + self.visit(&o.right)
    }

    fn visit_projection(&mut self, o: &'a Projection<Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_union(&mut self, o: &'a Union<Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.left) + self.visit(&o.right)
    }

    fn visit_filter(&mut self, o: &'a Filter<Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_minus(&mut self, o: &'a Minus<M, Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.left) + self.visit(&o.right)
    }

    fn visit_offset(&mut self, o: &'a Offset<Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_limit(&mut self, o: &'a Limit<L, Operation<'a, S, J, M, L>>) -> OperationMeta {
        self.visit(&o.operation)
    }
}
