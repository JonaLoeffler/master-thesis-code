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
            filters: self.filters + rhs.filters,
            disjunct_joins: self.disjunct_joins + rhs.disjunct_joins,
        }
    }
}

impl<'a> OperationVisitor<'a, OperationMeta> for Meta {
    fn visit_scan(&mut self, _o: &'a Scan) -> OperationMeta {
        OperationMeta {
            scans: 1,
            ..Default::default()
        }
    }

    fn visit_join(&mut self, o: &'a Join<Operation<'a>>) -> OperationMeta {
        let meta = OperationMeta {
            joins: 1,
            disjunct_joins: usize::from(o.join_vars().is_empty()),
            ..Default::default()
        };

        meta + self.visit(&o.left) + self.visit(&o.right)
    }

    fn visit_projection(&mut self, o: &'a Projection<Operation<'a>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_union(&mut self, o: &'a Union<Operation<'a>>) -> OperationMeta {
        self.visit(&o.left) + self.visit(&o.right)
    }

    fn visit_filter(&mut self, o: &'a Filter<Operation<'a>>) -> OperationMeta {
        let meta = OperationMeta {
            filters: 1,
            ..Default::default()
        };

        meta + self.visit(&o.operation)
    }

    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_minus(&mut self, o: &'a Minus<Operation<'a>>) -> OperationMeta {
        self.visit(&o.left) + self.visit(&o.right)
    }

    fn visit_offset(&mut self, o: &'a Offset<Operation<'a>>) -> OperationMeta {
        self.visit(&o.operation)
    }

    fn visit_limit(&mut self, o: &'a Limit<Operation<'a>>) -> OperationMeta {
        self.visit(&o.operation)
    }
}
