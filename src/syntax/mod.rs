use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash};

pub mod database;
pub(crate) mod expand;
pub mod query;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub enum Iri {
    IRIREF(String),
    PrefixedName(PrefixedName),
}

impl Iri {
    pub fn new(iri: String) -> Self {
        let mut iri = iri;
        if iri.ends_with(" .") {
            iri = iri.replace(" .", "");
        }

        if iri.starts_with('<') && iri.ends_with('>') {
            Self::IRIREF(iri)
        } else if iri.contains(':') {
            let mut iter = iri.split(':');
            let ns: String = iter.next().unwrap().into();
            let local: String = iter.next().unwrap().into();
            let expansion = None;

            Self::PrefixedName(PrefixedName {
                ns,
                local,
                expanded: expansion,
            })
        } else {
            panic!("Cannot create IRI from \'{iri}\'!")
        }
    }
}

impl From<String> for Iri {
    fn from(s: String) -> Self {
        Iri::new(s)
    }
}

impl From<&str> for Iri {
    fn from(s: &str) -> Self {
        Iri::new(s.to_string())
    }
}

impl PartialEq for Iri {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Iri::IRIREF(iri1) => match other {
                Iri::IRIREF(iri2) => iri1 == iri2,
                Iri::PrefixedName(name) => Some(iri1.to_string()) == name.expanded,
            },
            Iri::PrefixedName(name1) => match other {
                Iri::IRIREF(iri) => name1.expanded == Some(iri.to_string()),
                Iri::PrefixedName(name2) => name1.ns == name2.ns && name1.local == name2.local,
            },
        }
    }
}

impl Display for Iri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Iri::IRIREF(s) => f.write_str(s),
            Iri::PrefixedName(name) => f.write_str(&format!("{}:{}", name.ns, name.local)),
        }
    }
}

impl Hash for Iri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Iri::IRIREF(iri) => iri.hash(state),
            Iri::PrefixedName(name) => name.hash(state),
        }
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct PrefixedName {
    ns: String,
    local: String,
    expanded: Option<String>,
}

impl Hash for PrefixedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(expanded) = &self.expanded {
            expanded.hash(state);
        } else {
            self.ns.hash(state);
            self.local.hash(state);
        }
    }
}

impl PartialEq for PrefixedName {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(selfexpanded), Some(otherexpanded)) = (&self.expanded, &other.expanded) {
            selfexpanded.eq(otherexpanded)
        } else {
            self.ns.eq(&other.ns) && self.local.eq(&other.local)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Literal {
    value: String,
    pub(crate) parsed: Option<f64>,
    datatype: Option<String>,
    language: Option<String>,
}

impl Literal {
    pub fn new(raw: String) -> Self {
        let mut value = raw.clone();
        let mut datatype = None;
        let mut language = None;

        if value.contains("^^") {
            let mut split = raw.split("^^");

            value = split.next().unwrap().to_string();
            datatype = split.next().map(|v| v.to_string());
        }

        if value.contains('@') {
            let mut split = raw.split('@');

            value = split.next().unwrap().to_string();
            language = split.next().map(|v| v.to_string());
        }

        let parsed = value.replace('\"', "").parse().ok();

        Self {
            value,
            parsed,
            datatype,
            language,
        }
    }
}

impl Eq for Literal {}
impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parsed) = self.parsed {
            f.write_str(&format!("{parsed}"))
        } else {
            f.write_str(&self.value.to_string())
        }
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::new(value)
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Literal::new(value.to_string())
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Some(left) = self.parsed {
            if let Some(right) = other.parsed {
                return left.partial_cmp(&right);
            }
        }

        None
    }
}
