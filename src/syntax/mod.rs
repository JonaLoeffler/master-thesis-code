use std::{collections::HashMap, error::Error, fmt::Display, hash::Hash};

pub mod database;
pub mod query;

#[derive(Debug)]
pub enum ExpandError {
    PrefixNotFound,
}

impl Error for ExpandError {}

impl Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::PrefixNotFound => f.write_str("Prefix not found"),
        }
    }
}

trait Expand {
    type Expandable;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Self::Expandable, ExpandError>;
}

#[derive(Debug, Clone, Eq)]
pub enum Iri {
    IRIREF(String),
    PrefixedName(PrefixedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixedName {
    ns: String,
    local: String,
    expanded: Option<String>,
}

impl Iri {
    pub fn new(iri: String) -> Self {
        if iri.starts_with("<") && iri.ends_with(">") {
            Self::IRIREF(iri)
        } else if iri.contains(":") {
            let mut iter = iri.split(":");
            let ns: String = iter.next().unwrap().into();
            let local: String = iter.next().unwrap().into();
            let expansion = None;

            Self::PrefixedName(PrefixedName {
                ns,
                local,
                expanded: expansion,
            })
        } else {
            panic!("Cannot create IRI! from {}", iri)
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

impl Expand for Iri {
    type Expandable = Iri;

    fn expand(self, prefixes: &HashMap<String, String>) -> Result<Iri, ExpandError> {
        match self {
            Self::IRIREF(s) => Ok(Self::IRIREF(s)),
            Self::PrefixedName(name) => {
                let expanded = if let Some(expansion) = prefixes.get(&name.ns) {
                    Some(format!("{}{}>", expansion.replace(">", ""), name.local))
                } else {
                    None
                };

                Ok(Self::PrefixedName(PrefixedName {
                    ns: name.ns,
                    local: name.local,
                    expanded,
                }))
            }
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
