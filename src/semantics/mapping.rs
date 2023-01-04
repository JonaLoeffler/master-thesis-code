use core::fmt;
use std::collections::{btree_map::Keys, BTreeMap};

use itertools::Itertools;

use crate::syntax::{database, query};

pub(crate) type MappingSet = Vec<Mapping>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Mapping {
    pub(crate) items: BTreeMap<query::Variable, database::Object>,
}

impl Mapping {
    pub(crate) fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, v: &query::Variable) -> Option<&database::Object> {
        self.items.get(v)
    }

    pub(crate) fn insert(
        &mut self,
        k: query::Variable,
        v: database::Object,
    ) -> Option<database::Object> {
        self.items.insert(k, v)
    }

    pub(crate) fn contains_key(&self, v: &&query::Variable) -> bool {
        self.items.contains_key(v)
    }

    pub(crate) fn keys(&self) -> Keys<query::Variable, database::Object> {
        self.items.keys()
    }

    pub(crate) fn hash_map_key(&self, vars: &query::Variables) -> String {
        vars.iter().filter_map(|v| self.get(v)).join("")
    }

    /**
     * Check whether two mappings are compatible.
     *
     * They are not compatible if they contain the same variable with different values
     */
    pub(crate) fn compatible(&self, other: &Self) -> bool {
        for key in self.keys().filter(|key| other.contains_key(key)) {
            if let Some(x) = self.get(key) {
                if let Some(y) = other.get(key) {
                    if x != y {
                        // log::debug!("Incompatible on {:?}", key);

                        return false;
                    }
                }
            }
        }

        true
    }

    pub(crate) fn satisfies(&self, r: &query::Condition) -> bool {
        match r {
            query::Condition::Equals(o1, o2) => match o1 {
                query::Object::L(l1) => match o2 {
                    query::Object::L(l2) => l1 == l2,
                    query::Object::I(_) => false,
                    query::Object::V(_) => false,
                },
                query::Object::I(u1) => match o2 {
                    query::Object::L(_) => false,
                    query::Object::I(u2) => u1 == u2,
                    query::Object::V(_) => false,
                },
                query::Object::V(v1) => match o2 {
                    query::Object::L(l) => {
                        self.get(&v1) == Some(&database::Object::L(l.to_string()))
                    }
                    query::Object::I(u) => {
                        self.get(&v1) == Some(&database::Object::I(u.to_owned()))
                    }
                    query::Object::V(v2) => self.get(&v1) == self.get(&v2),
                },
            },
            query::Condition::Bound(v) => self.contains_key(&v),
            query::Condition::Not(c) => !self.satisfies(c),
            query::Condition::And(c1, c2) => self.satisfies(c1) && self.satisfies(c2),
            query::Condition::Or(c1, c2) => self.satisfies(c1) || self.satisfies(c2),
        }
    }
}

impl FromIterator<(query::Variable, database::Object)> for Mapping {
    fn from_iter<T: IntoIterator<Item = (query::Variable, database::Object)>>(iter: T) -> Self {
        let mut result = Self::new();

        for (v, o) in iter {
            result.insert(v, o);
        }

        result
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .items
            .iter()
            .map(|(k, v)| format!("{} -> {}", k.name, v));

        f.write_str(&itertools::Itertools::intersperse(s, ", ".to_string()).collect::<String>())
    }
}
