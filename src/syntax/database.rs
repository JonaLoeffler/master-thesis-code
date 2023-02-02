use iter_progress::ProgressableIter;
use itertools::Itertools;
use ndhistogram::{axis::Uniform, sparsehistogram, AxesTuple, HashHistogram, Histogram};
use rand::{seq::IteratorRandom, thread_rng};
use ron::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    hash::Hash,
    io::Write,
    path::Path,
};

use crate::semantics::{
    evaluate,
    options::{EvalOptions, Optimizer},
};
use crate::syntax::query::{self, Expression, Query, Type::SelectQuery, Variable, Variables};

use super::{Iri, Literal};

#[derive(Debug)]
pub struct Database {
    triples: Vec<Triple>,
    summary: Summary,
}

impl Database {
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
            summary: Summary::new(),
        }
    }

    pub fn add(&mut self, triple: Triple) {
        self.triples.push(triple);
    }

    pub fn triples(&self) -> &Vec<Triple> {
        &self.triples
    }

    pub fn summary(&self) -> &Summary {
        &self.summary
    }

    pub fn sample(&self, n: usize) -> Self {
        log::warn!("Sample N: {n}");

        Self {
            triples: self
                .triples
                .iter()
                .cloned()
                .choose_multiple(&mut thread_rng(), n),
            summary: self.summary.clone(),
        }
    }

    pub fn build_statistics(&mut self, database_path: &Path) -> Result<()> {
        let mut summary_path = database_path.to_path_buf();

        summary_path.set_extension("ron");

        if summary_path.exists() {
            log::info!("Reading database summary from file...");
            let ron = fs::read_to_string(summary_path).expect("Unable to read file");

            self.summary = ron::from_str(&ron)?;
            log::info!("Done!");
        } else {
            log::info!("Building database statistics...");

            self.summary = Summary::new();

            for (state, triple) in self.triples.iter().progress() {
                state.do_every_n_sec(1., |s| {
                    log::info!("Building database statistics, {:.2} per sec.", s.rate());
                });

                self.summary.update(triple);
            }

            log::info!("Computing joined triple pattern stats...");

            self.build_joined_statistics();

            log::info!("Database statistics done, {} triples", self.triples.len());

            let ron = ron::to_string(&self.summary)?;
            let mut file = File::create(summary_path)?;
            writeln!(file, "{ron}")?;
        }

        Ok(())
    }

    fn build_joined_statistics(&mut self) {
        let n = 10000.0_f64.max(self.triples.len() as f64 * 0.01).floor();

        let sample = self.sample(n as usize);

        log::info!(
            "Found {} predicates, computing {} combinations",
            self.summary.p.len(),
            self.summary.p.len() * self.summary.p.len()
        );

        self.summary.s_p = self
            .summary
            .p
            .iter()
            .cartesian_product(self.summary.p.iter())
            .progress()
            .map(|(state, (p1, p2))| {
                state.do_every_n_sec(1., |s| {
                    log::info!(
                        "{:.2}% done with join stats, {:.2} per sec.",
                        s.percent().unwrap(),
                        s.rate()
                    );
                });

                let query = Query {
                    prologue: HashMap::new(),
                    kind: SelectQuery(
                        Variables::new(vec![Variable::new("?X".to_owned())]),
                        Expression::And(
                            Box::new(Expression::Triple(
                                Box::new(query::Subject::V(Variable::new("?X".to_owned()))),
                                Box::new(match p1 {
                                    Predicate::I(i) => query::Predicate::I(i.clone()),
                                }),
                                Box::new(query::Object::V(Variable::new("?Y".to_owned()))),
                            )),
                            Box::new(Expression::Triple(
                                Box::new(query::Subject::V(Variable::new("?X".to_owned()))),
                                Box::new(match p2 {
                                    Predicate::I(i) => query::Predicate::I(i.clone()),
                                }),
                                Box::new(query::Object::V(Variable::new("?Z".to_owned()))),
                            )),
                        ),
                        query::SolutionModifier::default(),
                    ),
                };

                let opts = EvalOptions::default()
                    .with_log(false)
                    .with_optimizer(Optimizer::Off);

                let mut s_p = evaluate(&sample, query, Some(opts)).unwrap().size() as f64;

                // Normalize by sample size
                s_p *= self.triples.len() as f64 / n;

                ((p1.to_owned(), p2.to_owned()), s_p.ceil() as usize)
            })
            .collect::<HashMap<(Predicate, Predicate), usize>>();
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Triple> for Database {
    fn from_iter<T: IntoIterator<Item = Triple>>(iter: T) -> Self {
        log::info!("Building database...");

        let mut db = Database::new();

        for (state, i) in iter.into_iter().progress() {
            state.do_every_n_sec(1., |s| {
                log::info!("Building database, {:.2} per sec.", s.rate());
            });

            db.add(i);
        }

        db
    }
}

impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for triple in self.triples.iter() {
            f.write_fmt(format_args!(
                "{} {} {} .\n",
                triple.subject, triple.predicate, triple.object,
            ))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Triple {
    pub subject: Subject,
    pub predicate: Predicate,
    pub object: Object,
}

impl Triple {
    pub fn new(subject: Subject, predicate: Predicate, object: Object) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Subject {
    B,
    I(Iri),
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::B => Ok(f.write_str("()")?),
            Subject::I(u) => u.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    I(Iri),
}

impl Predicate {
    #[allow(dead_code)]
    fn is_numeric_datatype(&self) -> bool {
        let s = match self {
            Predicate::I(i) => i.to_string(),
        };

        println!("{s:#?}");

        vec![
            "<http://dbpedia.org/datatype/cubicCentimetre>",
            "<http://dbpedia.org/datatype/hertz>",
            "<http://dbpedia.org/datatype/hour>",
            "<http://dbpedia.org/datatype/mile>",
            "<http://dbpedia.org/datatype/minute>",
            "<http://dbpedia.org/datatype/perCent>",
            "<http://dbpedia.org/datatype/second>",
            "<http://www.w3.org/2001/XMLSchema#decimal>",
            "<http://www.w3.org/2001/XMLSchema#double>",
            "<http://www.w3.org/2001/XMLSchema#float>",
            "<http://www.w3.org/2001/XMLSchema#gMonthDay>",
            "<http://www.w3.org/2001/XMLSchema#gYear>",
            "<http://www.w3.org/2001/XMLSchema#integer>",
            "<http://www.w3.org/2001/XMLSchema#nonNegativeInteger>",
        ]
        .contains(&s.as_str())
    }
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::I(u) => u.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Object {
    B,
    L(Literal),
    I(Iri),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Object::B => f.write_str("()"),
            Object::L(l) => f.write_str(&format!("{l}")),
            Object::I(u) => f.write_str(&format!("{u}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Total number of triples
    t: usize,

    /// Distinct subjects
    r: HashSet<Subject>,

    /// Number of triples per predicate
    t_p: HashMap<Predicate, usize>,

    /// Number of triples per predicate and object
    pub o_c: HashMap<Predicate, HashMap<Object, usize>>,

    /// Distinct predicates
    p: HashSet<Predicate>,

    /// Joined Triple Upper Bounds
    s_p: HashMap<(Predicate, Predicate), usize>,

    /// Sparse Histograms for predicates with numeric values
    p_l: HashMap<Predicate, HashHistogram<AxesTuple<(Uniform,)>, f64>>,
}

impl Summary {
    pub fn t(&self) -> f64 {
        let tmp: u32 = self.t.try_into().unwrap();
        tmp as f64
    }

    pub fn r(&self) -> f64 {
        let tmp: u32 = self.r.len().try_into().unwrap();
        tmp as f64
    }

    pub fn t_p(&self, p: &Predicate) -> f64 {
        if let Some(count) = self.t_p.get(p) {
            *count as f64
        } else {
            0.0
        }
    }

    pub fn o_c(&self, p: &Predicate, o: &Object) -> f64 {
        let result = if let Some(predmap) = self.o_c.get(p) {
            if let Some(count) = predmap.get(o) {
                *count as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        result
    }

    pub fn s_p(&self, p1: Predicate, p2: Predicate) -> f64 {
        if let Some(val) = self.s_p.get(&(p1, p2)) {
            *val as f64
        } else {
            0.0
        }
    }

    pub fn p_l(&self, p: &Predicate, lower: Option<f64>, upper: Option<f64>) -> f64 {
        if let (None, None) = (lower, upper) {
            return 1.0;
        }

        if let Some(hist) = self.p_l.get(p) {
            let has_value_in_histogram = hist.iter().any(|b| b.value > &0.0);

            log::trace!("Lower: {:?}, Upper {:?}", lower, upper);

            let factor = if has_value_in_histogram {
                let inside_bounds = hist
                    .iter()
                    .filter(|b| match b.bin.end() {
                        // The bin end must be larger than the lower bound
                        Some(bin_end) => bin_end > lower.unwrap_or(-f64::INFINITY),
                        // If there is no bin end, we are in the overflow bin which is always
                        // considered larger than the lower bound
                        None => true,
                    })
                    .filter(|b| match b.bin.start() {
                        // The bin start must be smaller than the upper bound
                        Some(bin_start) => bin_start < upper.unwrap_or(f64::INFINITY),
                        // If there is no bin start, we are in the underflow bin which is always
                        // considered smaller than the lower bound
                        None => true,
                    })
                    .map(|b| b.value)
                    .sum::<f64>();

                inside_bounds / self.t_p(p)
            } else {
                1.0
            };

            log::trace!("Factor for predicate {p} {factor}");

            return factor;
        }

        1.0
    }

    pub fn new() -> Summary {
        Summary {
            t: 0,
            r: HashSet::new(),
            p: HashSet::new(),
            t_p: HashMap::new(),
            o_c: HashMap::new(),
            s_p: HashMap::new(),
            p_l: HashMap::new(),
        }
    }

    fn update(&mut self, triple: &Triple) {
        // update T
        self.t += 1;

        // Update R
        self.r.insert(triple.subject.to_owned());

        // Update P
        self.p.insert(triple.predicate.to_owned());

        // Update T_P
        if let Some(count) = self.t_p.get_mut(&triple.predicate) {
            *count += 1;
        } else {
            self.t_p.insert(triple.predicate.to_owned(), 1);
        }

        // Update O_c
        if let Some(predmap) = self.o_c.get_mut(&triple.predicate) {
            if let Some(count) = predmap.get_mut(&triple.object) {
                *count += 1;
            } else {
                predmap.insert(triple.object.to_owned(), 1);
            }
        } else {
            self.o_c.insert(
                triple.predicate.to_owned(),
                HashMap::from([(triple.object.to_owned(), 1)]),
            );
        }

        // Update p_l
        if let Some(hist) = self.p_l.get_mut(&triple.predicate) {
            if let Object::L(literal) = &triple.object {
                if let Some(value) = literal.parsed {
                    if !value.is_nan() {
                        hist.fill(&value);
                    }
                }
            }
        } else {
            self.p_l.insert(
                triple.predicate.to_owned(),
                sparsehistogram!(Uniform::new(200_000, -1_000_000.0, 1_000_000.0)),
            );
        }
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Database Statistics:\n")?;
        f.write_str(&format!("T: {}\n", self.t()))?;
        f.write_str(&format!("R: {}\n", self.r()))?;
        f.write_str(&format!("T_P: {}\n", self.t_p.len()))?;
        f.write_str(&format!("O_c: {} ", self.o_c.len()))?;

        let iter = self.o_c.values().map(|v| v.len().to_string());
        let res = Itertools::intersperse(iter, ", ".to_string());
        f.write_str(&format!("({})", res.collect::<String>()))?;
        f.write_str("\n")?;

        f.write_str(&format!("S_P: {}\n", self.s_p.len()))?;

        f.write_str(&format!(
            "P_L: {}\n",
            self.p_l
                .iter()
                .filter(|(_, h)| h.values().next().is_some())
                .count()
        ))?;

        Ok(())
    }
}
