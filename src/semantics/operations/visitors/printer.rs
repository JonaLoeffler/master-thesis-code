use crate::{
    semantics::{
        operations::{
            filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, minus::Minus,
            offset::Offset, projection::Projection, scan::Scan, union::Union, Operation,
            OperationVisitor,
        },
        selectivity::SelectivityEstimator,
    },
    syntax::query,
};

use super::bound::BoundVars;

pub struct Printer<'a> {
    bound: bool,
    join: bool,
    bgp: bool,
    estimator: Option<SelectivityEstimator<'a>>,
}

impl<'a> Default for Printer<'a> {
    fn default() -> Self {
        Self {
            estimator: None,
            bound: true,
            join: true,
            bgp: true,
        }
    }
}

impl<'a> Printer<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_bound(self, bound: bool) -> Self {
        Self { bound, ..self }
    }

    pub fn with_join(self, join: bool) -> Self {
        Self { join, ..self }
    }

    pub fn with_estimator(self, estimator: Option<SelectivityEstimator<'a>>) -> Self {
        Self { estimator, ..self }
    }

    pub fn with_bgp(self, bgp: bool) -> Self {
        Self { bgp, ..self }
    }
}

impl<'a, 'b> OperationVisitor<'a, String> for Printer<'b> {
    fn visit_scan(&mut self, o: &'a Scan) -> String {
        let bound: Option<String> = if self.bound {
            let bound = BoundVars::new()
                .visit_scan(o)
                .iter()
                .cloned()
                .collect::<query::Variables>();

            Some(format!("Bound: {bound}"))
        } else {
            None
        };

        let bgp: Option<String> = if self.bgp {
            Some(format!(
                "BGP: {{ {} {} {} }}",
                o.subject, o.predicate, o.object,
            ))
        } else {
            None
        };

        let selectivity: Option<String> = self.estimator.as_ref().map(|estimator| {
            format!(
                "{}: {:1.2e}",
                estimator,
                estimator.selectivity(o).unwrap_or(f64::NAN)
            )
        });

        vec![Some("SCAN".to_owned()), selectivity, bound, bgp]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn visit_join(&mut self, o: &'a Join<Operation<'a>>) -> String {
        let bound: Option<String> = if self.bound {
            let bound = BoundVars::new()
                .visit_join(o)
                .iter()
                .cloned()
                .collect::<query::Variables>();

            Some(format!("Bound: {bound}"))
        } else {
            None
        };

        let join: Option<String> = if self.join {
            Some(format!("Join: {}", o.join_vars,))
        } else {
            None
        };

        let selectivity: Option<String> = self.estimator.as_ref().map(|estimator| {
            format!(
                "{}: {:1.2e}",
                estimator,
                estimator.selectivity(o).unwrap_or(f64::NAN)
            )
        });

        let line = vec![Some("JOIN".to_owned()), selectivity, bound, join]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join(" ");

        vec![line, self.visit(&o.left), self.visit(&o.right)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_projection(&mut self, o: &'a Projection<Operation<'a>>) -> String {
        vec![format!("PROJECTION {}", o.vars), self.visit(&o.operation)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_union(&mut self, o: &'a Union<Operation<'a>>) -> String {
        vec![format!("UNION"), self.visit(&o.left), self.visit(&o.right)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_filter(&mut self, o: &'a Filter<Operation<'a>>) -> String {
        vec![format!("FILTER {}", o.condition), self.visit(&o.operation)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_leftjoin(&mut self, o: &'a LeftJoin<Operation<'a>>) -> String {
        vec![format!("LEFTJOIN"), self.visit(&o.operation)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_minus(&mut self, o: &'a Minus<Operation<'a>>) -> String {
        vec![format!("MINUS"), self.visit(&o.left), self.visit(&o.right)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_offset(&mut self, o: &'a Offset<Operation<'a>>) -> String {
        vec![format!("OFFSET {}", o.offset), self.visit(&o.operation)]
            .join("\n")
            .replace('\n', "\n  ")
    }

    fn visit_limit(&mut self, o: &'a Limit<Operation<'a>>) -> String {
        vec![format!("LIMIT {}", o.limit), self.visit(&o.operation)]
            .join("\n")
            .replace('\n', "\n  ")
    }
}
