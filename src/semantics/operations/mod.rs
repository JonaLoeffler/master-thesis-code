pub(super) mod filter;
pub(super) mod join;
pub(super) mod leftjoin;
pub(super) mod limit;
pub(super) mod minus;
pub(super) mod offset;
pub(super) mod projection;
pub(super) mod scan;
pub(super) mod union;
pub(super) mod visitors;

use core::fmt;

use crate::syntax::database;

use self::{
    filter::Filter,
    join::{CollJoin, IterJoin, Join},
    leftjoin::LeftJoin,
    limit::{CollLimit, IterLimit, Limit},
    minus::{CollMinus, IterMinus, Minus},
    offset::Offset,
    projection::Projection,
    scan::{CollScan, IterScan, Scan},
    union::Union,
    visitors::meta::Meta,
};

use super::{
    mapping::{Mapping, MappingSet},
    results::OperationMeta,
    selectivity::{Selectivity, SelectivityError},
};

pub(crate) trait Execute {
    fn execute(&self) -> MappingSet;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Operation<'a, S, J, M, L> {
    Scan(Scan<'a, S, J, M, L>),
    Join(Join<J, Self>),
    Projection(Projection<Self>),
    Union(Union<Self>),
    Filter(Filter<Self>),
    LeftJoin(LeftJoin<Self>),
    Minus(Minus<M, Self>),
    Offset(Offset<Self>),
    Limit(Limit<L, Self>),
}

impl<'a, S, J, M, L> Operation<'a, S, J, M, L> {
    pub(crate) fn meta(&self) -> OperationMeta {
        Meta::new().visit(self)
    }
}

impl<'a> Execute for Operation<'a, CollScan, CollJoin, CollMinus, CollLimit> {
    fn execute(&self) -> MappingSet {
        match self {
            Operation::Scan(s) => s.execute(),
            Operation::Join(j) => j.execute(),
            Operation::Projection(p) => p.execute(),
            Operation::Union(u) => u.execute(),
            Operation::Filter(f) => f.execute(),
            Operation::LeftJoin(o) => o.execute(),
            Operation::Minus(m) => m.execute(),
            Operation::Offset(o) => o.execute(),
            Operation::Limit(l) => l.execute(),
        }
    }
}

impl<'a> Iterator for Operation<'a, IterScan<'a>, IterJoin, IterMinus, IterLimit> {
    type Item = Mapping;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Operation::Scan(s) => s.next(),
            Operation::Join(j) => j.next(),
            Operation::Projection(p) => p.next(),
            Operation::Union(u) => u.next(),
            Operation::Filter(f) => f.next(),
            Operation::LeftJoin(o) => o.next(),
            Operation::Minus(m) => m.next(),
            Operation::Offset(o) => o.next(),
            Operation::Limit(l) => l.next(),
        }
    }
}

impl<'a, S, J, M, L> fmt::Display for Operation<'a, S, J, M, L> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Scan(s) => s.fmt(fmt),
            Operation::Join(j) => j.fmt(fmt),
            Operation::Projection(p) => p.fmt(fmt),
            Operation::Union(u) => u.fmt(fmt),
            Operation::Filter(f) => f.fmt(fmt),
            Operation::LeftJoin(o) => o.fmt(fmt),
            Operation::Minus(m) => m.fmt(fmt),
            Operation::Offset(o) => o.fmt(fmt),
            Operation::Limit(l) => l.fmt(fmt),
        }
    }
}

impl<'a, S, J, M, L> Selectivity for Operation<'a, S, J, M, L> {
    fn sel_vc(&self) -> Result<f32, SelectivityError> {
        match self {
            Operation::Scan(s) => s.sel_vc(),
            Operation::Join(j) => j.sel_vc(),
            Operation::Projection(p) => p.sel_vc(),
            Operation::Union(u) => u.sel_vc(),
            Operation::Filter(f) => f.sel_vc(),
            Operation::LeftJoin(l) => l.sel_vc(),
            Operation::Minus(m) => m.sel_vc(),
            Operation::Offset(o) => o.sel_vc(),
            Operation::Limit(l) => l.sel_vc(),
        }
    }

    fn sel_vcp(&self) -> Result<f32, SelectivityError> {
        match self {
            Operation::Scan(s) => s.sel_vcp(),
            Operation::Join(j) => j.sel_vcp(),
            Operation::Projection(p) => p.sel_vcp(),
            Operation::Union(u) => u.sel_vcp(),
            Operation::Filter(f) => f.sel_vcp(),
            Operation::LeftJoin(l) => l.sel_vcp(),
            Operation::Minus(m) => m.sel_vcp(),
            Operation::Offset(o) => o.sel_vcp(),
            Operation::Limit(l) => l.sel_vcp(),
        }
    }

    fn sel_pf(&self, summary: &database::Summary) -> Result<f32, SelectivityError> {
        match self {
            Operation::Scan(s) => s.sel_pf(summary),
            Operation::Join(j) => j.sel_pf(summary),
            Operation::Projection(p) => p.sel_pf(summary),
            Operation::Union(u) => u.sel_pf(summary),
            Operation::Filter(f) => f.sel_pf(summary),
            Operation::LeftJoin(l) => l.sel_pf(summary),
            Operation::Minus(m) => m.sel_pf(summary),
            Operation::Offset(o) => o.sel_pf(summary),
            Operation::Limit(l) => l.sel_pf(summary),
        }
    }

    fn sel_pfj(&self, summary: &database::Summary) -> Result<f32, SelectivityError> {
        match self {
            Operation::Scan(s) => s.sel_pfj(summary),
            Operation::Join(j) => j.sel_pfj(summary),
            Operation::Projection(p) => p.sel_pfj(summary),
            Operation::Union(u) => u.sel_pfj(summary),
            Operation::Filter(f) => f.sel_pfj(summary),
            Operation::LeftJoin(l) => l.sel_pfj(summary),
            Operation::Minus(m) => m.sel_pfj(summary),
            Operation::Offset(o) => o.sel_pfj(summary),
            Operation::Limit(l) => l.sel_pfj(summary),
        }
    }
}

pub(super) trait OperationVisitor<'a, S, J, M, L, R> {
    fn visit(&mut self, o: &'a Operation<'a, S, J, M, L>) -> R {
        match o {
            Operation::Scan(s) => self.visit_scan(s),
            Operation::Join(j) => self.visit_join(j),
            Operation::Projection(p) => self.visit_projection(p),
            Operation::Union(u) => self.visit_union(u),
            Operation::Filter(f) => self.visit_filter(f),
            Operation::LeftJoin(l) => self.visit_leftjoin(l),
            Operation::Minus(m) => self.visit_minus(m),
            Operation::Offset(o) => self.visit_offset(o),
            Operation::Limit(l) => self.visit_limit(l),
        }
    }

    fn visit_scan(&mut self, o: &'a Scan<S, J, M, L>) -> R;
    fn visit_join(&mut self, o: &'a Join<J, Operation<'a, S, J, M, L>>) -> R;
    fn visit_projection(&mut self, o: &'a Projection<Operation<'a, S, J, M, L>>) -> R;
    fn visit_union(&mut self, o: &'a Union<Operation<'a, S, J, M, L>>) -> R;
    fn visit_filter(&mut self, o: &'a Filter<Operation<'a, S, J, M, L>>) -> R;
    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a, S, J, M, L>>) -> R;
    fn visit_minus(&mut self, o: &'a Minus<M, Operation<'a, S, J, M, L>>) -> R;
    fn visit_offset(&mut self, o: &'a Offset<Operation<'a, S, J, M, L>>) -> R;
    fn visit_limit(&mut self, o: &'a Limit<L, Operation<'a, S, J, M, L>>) -> R;
}
