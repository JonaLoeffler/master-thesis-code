use clap::{Args, Parser as ArgumentParser, Subcommand};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use std::{error::Error, time::Duration};

use thesis::{
    examples as ex,
    semantics::{
        self,
        options::{self, EvalOptions},
    },
    syntax::{database::Database, query::Query},
};

#[derive(ArgumentParser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Example(Example),
    Parse(Parse),
    Lubm(Lubm),
    Explore(Explore),
}

#[derive(Args)]
struct Example {
    /// The number of the LUBM query to evaluate
    #[arg(short, long)]
    number: Option<usize>,
    /// Whether to print full results
    #[arg(short, long)]
    print: bool,
    /// Whether to use the optimizer
    #[arg(short, long, value_enum, default_value_t = options::Optimizer::ARQPF)]
    optimizer: options::Optimizer,
    /// Which semantics implementation to use
    #[arg(short, long, value_enum, default_value_t = options::Semantics::Iterator)]
    semantics: options::Semantics,
    /// Whether to abort just before query execution
    #[arg(short, long)]
    dryrun: bool,
}

#[derive(Args)]
struct Lubm {
    /// The file to evaluate the LUBM queries on
    database_path: PathBuf,
    /// The number of the LUBM query to evaluate
    #[arg(short, long)]
    number: Option<usize>,
    /// Whether to print full results
    #[arg(short, long)]
    print: bool,
    /// Whether to use the optimizer
    #[arg(short, long, value_enum, default_value_t = options::Optimizer::ARQPF)]
    optimizer: options::Optimizer,
    /// Which semantics implementation to use
    #[arg(short, long, value_enum, default_value_t = options::Semantics::Iterator)]
    semantics: options::Semantics,
    /// Whether to abort just before query execution
    #[arg(short, long)]
    dryrun: bool,
}

#[derive(Args)]
struct Parse {
    /// The queries file to parse
    queries_path: PathBuf,
    /// The database file to parse
    database_path: PathBuf,
    /// The number of the query to evaluate
    #[arg(short, long)]
    number: Option<usize>,
    /// Whether to print full results
    #[arg(short, long)]
    print: bool,
    /// Whether to use the optimizer
    #[arg(short, long, value_enum, default_value_t = options::Optimizer::ARQPF)]
    optimizer: options::Optimizer,
    /// Which semantics implementation to use
    #[arg(short, long, value_enum, default_value_t = options::Semantics::Iterator)]
    semantics: options::Semantics,
    /// Whether to abort just before query execution
    #[arg(short, long)]
    dryrun: bool,
}

#[derive(Args)]
struct Explore {
    /// The queries file to parse
    queries_path: PathBuf,
    /// The database file to parse
    database_path: PathBuf,
    /// The number of the query to evaluate
    #[arg(short, long)]
    number: Option<usize>,
    #[arg(short, long)]
    dryrun: bool,
}

type ExitResult = Result<(), Box<dyn Error>>;

fn main() -> ExitResult {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Example(args) => example(args),
        Commands::Lubm(args) => lubm(args),
        Commands::Parse(args) => parse(args),
        Commands::Explore(args) => explore(args),
    }
}

fn lubm(args: &Lubm) -> ExitResult {
    let queries = vec![
        ("lubm1".to_string(), ex::lubm::query1()),
        ("lubm2".to_string(), ex::lubm::query2()),
        ("lubm3".to_string(), ex::lubm::query3()),
        ("lubm4".to_string(), ex::lubm::query4()),
        ("lubm5".to_string(), ex::lubm::query5()),
        ("lubm6".to_string(), ex::lubm::query6()),
        ("lubm7".to_string(), ex::lubm::query7()),
        ("lubm8".to_string(), ex::lubm::query8()),
        ("lubm9".to_string(), ex::lubm::query9()),
        ("lubm10".to_string(), ex::lubm::query10()),
        ("lubm11".to_string(), ex::lubm::query11()),
        ("lubm12".to_string(), ex::lubm::query12()),
        ("lubm13".to_string(), ex::lubm::query13()),
        ("lubm14".to_string(), ex::lubm::query14()),
    ];

    let filtered = queries
        .iter()
        .enumerate()
        .filter_map(|(i, q)| {
            if let Some(number) = args.number {
                if i + 1 == number.clamp(1, queries.len()) {
                    Some(q.clone())
                } else {
                    None
                }
            } else {
                Some(q.clone())
            }
        })
        .collect::<Vec<(String, Query)>>();

    let database = parse_database(&args.database_path)?;

    run_queries_on_db(
        filtered,
        database,
        Some(EvalOptions::new(
            args.semantics,
            args.optimizer,
            args.dryrun,
        )),
        args.print,
    )
}

fn parse(args: &Parse) -> ExitResult {
    let queries = fs::read_to_string(&args.queries_path)
        .expect("Should have been able to read this file")
        .split("\n\n")
        .into_iter()
        .map(|c| c.parse::<Query>())
        .collect::<Result<Vec<Query>, Box<dyn Error>>>()?;

    let filtered = queries
        .iter()
        .enumerate()
        .map(|(i, q)| (i, (format!("query{}", i + 1), q.clone())))
        .filter_map(|(i, q)| {
            if let Some(number) = args.number {
                if i + 1 == number.clamp(1, queries.len()) {
                    Some(q.clone())
                } else {
                    None
                }
            } else {
                Some(q.clone())
            }
        })
        .collect::<Vec<(String, Query)>>();

    run_queries_on_db(
        filtered,
        parse_database(&args.database_path)?,
        Some(EvalOptions::new(
            args.semantics,
            args.optimizer,
            args.dryrun,
        )),
        args.print,
    )
}

fn example(args: &Example) -> ExitResult {
    let queries = vec![
        ("query1".to_string(), ex::queries::example1()),
        ("query2".to_string(), ex::queries::example2()),
        ("query3".to_string(), ex::queries::example3()),
        ("query4".to_string(), ex::queries::example4()),
        ("query5".to_string(), ex::queries::example5()),
        ("query6".to_string(), ex::queries::example6()),
        ("query7".to_string(), ex::queries::example7()),
        ("query8".to_string(), ex::queries::example8()),
    ];

    let filtered = queries
        .iter()
        .cloned()
        .enumerate()
        .filter_map(|(i, q)| {
            if let Some(number) = args.number {
                if i + 1 == number.clamp(1, queries.len()) {
                    Some(q.clone())
                } else {
                    None
                }
            } else {
                Some(q.clone())
            }
        })
        .collect();

    run_queries_on_db(
        filtered,
        ex::databases::example1(),
        Some(EvalOptions::new(
            args.semantics,
            args.optimizer,
            args.dryrun,
        )),
        args.print,
    )
}

fn explore(args: &Explore) -> ExitResult {
    let queries = fs::read_to_string(&args.queries_path)
        .expect("Should have been able to read this file")
        .split("\n\n")
        .into_iter()
        .map(|c| c.parse::<Query>())
        .collect::<Result<Vec<Query>, Box<dyn Error>>>()?;

    let filtered = queries
        .iter()
        .enumerate()
        .map(|(i, q)| (i, (format!("{}", i + 1), q.clone())))
        .filter_map(|(i, q)| {
            if let Some(number) = args.number {
                if i + 1 == number.clamp(1, queries.len()) {
                    Some(q.clone())
                } else {
                    None
                }
            } else {
                Some(q.clone())
            }
        })
        .collect::<Vec<(String, Query)>>();

    let database = parse_database(&args.database_path)?;

    for (number, query) in filtered {
        let results = semantics::explore::explore(query, &database)?;

        for (i, result) in results.iter().enumerate() {
            println!(
                "explore,{},{:?},{},{:.8},{:?},{},{:?},{},\"{:?}\",{},{},{}",
                number,
                i,
                result.size(),
                result.duration().unwrap_or_default().as_secs_f32(),
                args.queries_path,
                queries.len(),
                args.database_path,
                database.triples().len(),
                result.optimizers(),
                result.operations().as_ref().unwrap().joins,
                result.operations().as_ref().unwrap().scans,
                result.operations().as_ref().unwrap().disjunct_joins,
            );
        }
    }

    Ok(())
}

fn parse_database(path: &PathBuf) -> Result<Database, Box<dyn Error>> {
    match path.extension().and_then(OsStr::to_str) {
        Some("nt") => Database::from_ntriples_str(
            &fs::read_to_string(path).expect("Should have been able to read this file"),
        ),
        Some("ttl") => Database::from_turtle_str(
            &fs::read_to_string(path).expect("Should have been able to read this file"),
        ),
        _ => panic!("Cannot parse database"),
    }
}

fn run_queries_on_db(
    queries: Vec<(String, Query)>,
    db: Database,
    opts: Option<EvalOptions>,
    print: bool,
) -> ExitResult {
    println!("{}", db.summary());
    println!(
        "Executing {} queries\n\n{} ",
        queries.len(),
        opts.to_owned().unwrap_or_default()
    );

    let now = Instant::now();
    let mut duration = Duration::default();

    for (name, query) in queries.into_iter() {
        let results = semantics::evaluate(&db, query, opts.to_owned())?;

        duration += results.duration().unwrap_or_default();

        println!("Total rows for {}: {}", name, results.size());

        if print {
            println!("{}", results);
        }
    }

    println!("Finished in {:.2?}", now.elapsed());
    println!("Without optimizations {}", duration.as_secs_f32());

    Ok(())
}
