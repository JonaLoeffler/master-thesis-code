pub mod explore;
mod mapping;
mod operations;
pub mod options;
mod results;
mod selectivity;

#[cfg(test)]
mod tests;

use crate::semantics::{
    operations::{
        visitors::{optimize::Optimize, planner::Planner},
        OperationVisitor,
    },
    options::{Optimizer, Semantics},
};

use crate::syntax::{
    database,
    query::{self, QueryVisitor},
};

use std::error::Error;
use std::fmt::Display;
use std::time::Instant;

use self::operations::{
    join::{CollJoin, IterJoin},
    limit::{CollLimit, IterLimit},
    minus::{CollMinus, IterMinus},
    scan::{CollScan, IterScan},
    Execute,
};
use self::{options::EvalOptions, results::QueryResult};

/**
* Evaluate a query on a database
*/
pub fn evaluate(
    db: &database::Database,
    query: query::Query,
    opts: Option<EvalOptions>,
) -> Result<QueryResult, Box<dyn Error>> {
    log::info!(
        "--- Evaluating query ---\n{} on {} triples",
        query,
        db.triples().len()
    );

    let opts = opts.unwrap_or_default();

    let optimizer = match opts.optimizer {
        Optimizer::Off => selectivity::SelectivityEstimator::Off,
        Optimizer::Random => selectivity::SelectivityEstimator::Random,
        Optimizer::Fixed => selectivity::SelectivityEstimator::Fixed,
        Optimizer::ARQPF => selectivity::SelectivityEstimator::ARQPF(db.summary()),
        Optimizer::ARQPFJ => selectivity::SelectivityEstimator::ARQPFJ(db.summary()),
        Optimizer::ARQVC => selectivity::SelectivityEstimator::ARQVC,
        Optimizer::ARQVCP => selectivity::SelectivityEstimator::ARQVCP,
    };

    let expanded = query.clone().expand()?;

    let now = Instant::now();

    let result = match opts.semantics {
        Semantics::Iterator => {
            let plan = Planner::iter(db).visit(&expanded);
            log::info!("--- Initial Query Plan ---\n{}\n", plan);

            let optimized = Optimize::iter(optimizer.clone()).visit(&plan)?;
            log::info!("--- Optimized Query Plan ---\n{}\n", optimized);

            let mut final_plan = Box::new(optimized.clone()) as Box<dyn PlanResult>;

            if opts.dryrun {
                QueryResult::dryrun()
            } else {
                match query.kind {
                    query::Type::SelectQuery(_, _, _) => final_plan.select(),
                    query::Type::AskQuery(_, _) => final_plan.ask(),
                }
            }
        }
        Semantics::Collection => {
            let plan = Planner::coll(db).visit(&expanded);
            log::info!("--- Initial Query Plan ---\n{}\n", plan);

            let optimized = Optimize::coll(optimizer.clone()).visit(&plan)?;
            log::info!("--- Optimized Query Plan ---\n{}\n", optimized);

            let mut final_plan = Box::new(optimized.clone()) as Box<dyn PlanResult>;

            if opts.dryrun {
                QueryResult::dryrun()
            } else {
                match query.kind {
                    query::Type::SelectQuery(_, _, _) => final_plan.select(),
                    query::Type::AskQuery(_, _) => final_plan.ask(),
                }
            }
        }
    };

    Ok(result.with_duration(now.elapsed()))
}

trait PlanResult: Display {
    fn ask(&mut self) -> QueryResult;
    fn select(&mut self) -> QueryResult;
}

impl<'a> PlanResult for operations::Operation<'a, IterScan<'a>, IterJoin, IterMinus, IterLimit> {
    fn ask(&mut self) -> QueryResult {
        QueryResult::ask(self.next().is_some())
    }

    fn select(&mut self) -> QueryResult {
        QueryResult::select(self.collect())
    }
}

impl<'a> PlanResult for operations::Operation<'a, CollScan, CollJoin, CollMinus, CollLimit> {
    fn ask(&mut self) -> QueryResult {
        QueryResult::ask(!self.execute().is_empty())
    }

    fn select(&mut self) -> QueryResult {
        QueryResult::select(self.execute())
    }
}
