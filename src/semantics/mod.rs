pub mod explore;
mod mapping;
mod operations;
pub mod options;
mod results;
mod selectivity;

#[cfg(test)]
mod tests;

use crate::{
    semantics::{
        operations::{
            visitors::{optimize::Optimize, planner::Planner},
            OperationVisitor,
        },
        options::Optimizer,
    },
    syntax::expand::Expand,
};

use crate::syntax::{
    database,
    query::{self, QueryVisitor},
};

use std::error::Error;
use std::time::Instant;

use self::{
    operations::visitors::condition::ConditionAnalyzer, options::EvalOptions, results::QueryResult,
};

/**
* Evaluate a query on a database
*/
pub fn evaluate(
    db: &database::Database,
    query: query::Query,
    opts: Option<EvalOptions>,
) -> Result<QueryResult, Box<dyn Error>> {
    let opts = opts.unwrap_or_default();

    if opts.log {
        log::warn!(
            "--- Evaluating query ---\n{} on {} triples",
            query,
            db.triples().len()
        );
    }

    let info = ConditionAnalyzer::new().visit(&query);
    let optimizer = match opts.optimizer {
        Optimizer::Off => selectivity::SelectivityEstimator::Off,
        Optimizer::Random => selectivity::SelectivityEstimator::Random,
        Optimizer::Fixed => selectivity::SelectivityEstimator::Fixed,
        Optimizer::Arqpf => selectivity::SelectivityEstimator::Arqpf(db.summary()),
        Optimizer::Arqpfc => selectivity::SelectivityEstimator::Arqpfc(db.summary(), &info),
        Optimizer::Arqpfj => selectivity::SelectivityEstimator::Arqpfj(db.summary()),
        Optimizer::Arqpfjc => selectivity::SelectivityEstimator::Arqpfjc(db.summary(), &info),
        Optimizer::Arqvc => selectivity::SelectivityEstimator::Arqvc,
        Optimizer::Arqvcp => selectivity::SelectivityEstimator::Arqvcp,
    };

    let expanded = Expand::new(query.prologue.clone()).visit(&query)?;

    let plan = Planner::new(db).visit(&expanded);

    if opts.log {
        log::warn!("--- Initial Query Plan ---\n{}\n", plan);
    }

    let now = Instant::now();

    let mut optimized = Optimize::new(optimizer.clone())
        .with_condition(opts.condition)
        .visit(&plan)?;

    if opts.log {
        log::warn!("--- Optimized Query Plan ---\n{}\n", optimized);
    }

    let optimization_duration = now.elapsed();

    let now = Instant::now();

    let result = if opts.dryrun {
        QueryResult::dryrun()
    } else {
        match query.kind {
            query::Type::SelectQuery(_, _, _) => QueryResult::select(optimized.collect()),
            query::Type::AskQuery(_, _) => QueryResult::ask(optimized.next().is_some()),
        }
    };

    Ok(result
        .with_run_duration(now.elapsed())
        .with_optimization_duration(optimization_duration))
}
