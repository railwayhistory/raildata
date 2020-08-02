
use std::{borrow, cmp, fmt, ops, str};
use std::str::FromStr;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::load::report::{Failed, PathReporter};
use crate::load::yaml::{FromYaml, Value};
use super::marked::Marked;


//------------ Key -----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Key(String);

impl Key {
    pub fn from_string(s: String) -> Result<Self, InvalidKey> {
        Ok(Key(s))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Marked<Key> {
    pub fn from_string(s: Marked<String>, _report: &mut PathReporter)
                       -> Result<Self, Failed> {
        Ok(s.map(Key))
    }
}


//--- Deref, AsRef, and Borrow

impl ops::Deref for Key {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<Key> for Key {
    fn as_ref(&self) -> &Self { 
        self
    }
}

impl borrow::Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}


//--- FromStr and FromYaml

impl str::FromStr for Key {
    type Err = InvalidKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Key(s.into()))
    }
}

impl<C> FromYaml<C> for Marked<Key> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(value.into_string(report)?.map(Key))
    }
}


//--- PartialOrd and Ord

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let mut left = self.0.split('.');
        let mut right = other.0.split('.');

        loop {
            match (left.next(), right.next()) {
                (None, None) => return cmp::Ordering::Equal,
                (None, Some(_)) => return cmp::Ordering::Less,
                (Some(_), None) => return cmp::Ordering::Greater,
                (Some(left), Some(right)) => {
                    let cmp = match (usize::from_str(left),
                                     usize::from_str(right)) {
                        (Ok(left), Ok(right)) => left.cmp(&right),
                        _ => left.cmp(right)
                    };
                    if cmp != cmp::Ordering::Equal {
                        return cmp
                    }
                }
            }
        }
    }
}


//--- Display

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}


//------------ InvalidKey ----------------------------------------------------

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid key")]
pub struct InvalidKey;

