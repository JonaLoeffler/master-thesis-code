#![allow(warnings)]
mod operation;
mod operation_planner;
mod operation_visitor;
mod query_ast;
mod query_visitor;

use crate::syntax::database::Object;
use std::collections::BTreeMap;

use self::query_ast::Variable;

type Mapping = BTreeMap<Variable, Object>;

type MappingSet = Vec<Mapping>;
