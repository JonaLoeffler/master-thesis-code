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
use std::collections::HashSet;

use crate::syntax::{database, query::Variable};

use self::{
    filter::Filter,
    join::Join,
    leftjoin::LeftJoin,
    limit::Limit,
    minus::Minus,
    offset::Offset,
    projection::Projection,
    scan::Scan,
    union::Union,
    visitors::{bound::BoundVars, condition::ConditionInfo, meta::Meta, printer::Printer},
};

use super::{
    mapping::Mapping,
    results::OperationMeta,
    selectivity::{Selectivity, SelectivityResult},
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum Operation<'a> {
    Scan(Scan<'a>),
    Join(Join<Self>),
    Projection(Projection<Self>),
    Union(Union<Self>),
    Filter(Filter<Self>),
    LeftJoin(LeftJoin<Self>),
    Minus(Minus<Self>),
    Offset(Offset<Self>),
    Limit(Limit<Self>),
}

impl<'a> Operation<'a> {
    pub(crate) fn meta(&self) -> OperationMeta {
        Meta::new().visit(self)
    }

    pub(crate) fn bound_vars(&self) -> HashSet<Variable> {
        BoundVars::new().visit(self)
    }
}

impl<'a> Iterator for Operation<'a> {
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

impl<'a> fmt::Display for Operation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Printer::new().visit(self))
    }
}

impl<'a> Selectivity for Operation<'a> {
    fn sel_vc(&self) -> SelectivityResult {
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

    fn sel_vcp(&self) -> SelectivityResult {
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

    fn sel_pf(&self, summary: &database::Summary) -> SelectivityResult {
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

    fn sel_pfc(&self, summary: &database::Summary, info: &ConditionInfo) -> SelectivityResult {
        match self {
            Operation::Scan(s) => s.sel_pfc(summary, info),
            Operation::Join(j) => j.sel_pfc(summary, info),
            Operation::Projection(p) => p.sel_pfc(summary, info),
            Operation::Union(u) => u.sel_pfc(summary, info),
            Operation::Filter(f) => f.sel_pfc(summary, info),
            Operation::LeftJoin(l) => l.sel_pfc(summary, info),
            Operation::Minus(m) => m.sel_pfc(summary, info),
            Operation::Offset(o) => o.sel_pfc(summary, info),
            Operation::Limit(l) => l.sel_pfc(summary, info),
        }
    }

    fn sel_pfj(&self, summary: &database::Summary) -> SelectivityResult {
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

    fn sel_pfjc(&self, summary: &database::Summary, info: &ConditionInfo) -> SelectivityResult {
        match self {
            Operation::Scan(s) => s.sel_pfjc(summary, info),
            Operation::Join(j) => j.sel_pfjc(summary, info),
            Operation::Projection(p) => p.sel_pfjc(summary, info),
            Operation::Union(u) => u.sel_pfjc(summary, info),
            Operation::Filter(f) => f.sel_pfjc(summary, info),
            Operation::LeftJoin(l) => l.sel_pfjc(summary, info),
            Operation::Minus(m) => m.sel_pfjc(summary, info),
            Operation::Offset(o) => o.sel_pfjc(summary, info),
            Operation::Limit(l) => l.sel_pfjc(summary, info),
        }
    }
}

pub(super) trait OperationVisitor<'a, R> {
    fn visit(&mut self, o: &'a Operation<'a>) -> R {
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

    fn visit_scan(&mut self, o: &'a Scan) -> R;
    fn visit_join(&mut self, o: &'a Join<Operation<'a>>) -> R;
    fn visit_projection(&mut self, o: &'a Projection<Operation<'a>>) -> R;
    fn visit_union(&mut self, o: &'a Union<Operation<'a>>) -> R;
    fn visit_filter(&mut self, o: &'a Filter<Operation<'a>>) -> R;
    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a>>) -> R;
    fn visit_minus(&mut self, o: &'a Minus<Operation<'a>>) -> R;
    fn visit_offset(&mut self, o: &'a Offset<Operation<'a>>) -> R;
    fn visit_limit(&mut self, o: &'a Limit<Operation<'a>>) -> R;
}
