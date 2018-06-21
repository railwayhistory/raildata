
use std::{fmt, str};
use ::load::report::{Failed, PathReporter};
use ::load::yaml::{FromYaml, Value};
use super::marked::Marked;


//------------ Key -----------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(String);

impl Key {
    pub fn from_string(s: String) -> Result<Self, InvalidKey> {
        Ok(Key(s))
    }
}

impl Marked<Key> {
    pub fn from_string(s: Marked<String>, _report: &mut PathReporter)
                       -> Result<Self, Failed> {
        Ok(s.map(Key))
    }
}

impl str::FromStr for Key {
    type Err = InvalidKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Key(s.into()))
    }
}

impl<C> FromYaml<C> for Marked<Key> {
    fn from_yaml(
        value: Value,
        _: &mut C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(value.into_string(report)?.map(Key))
    }
}


impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}


//------------ InvalidKey ----------------------------------------------------

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid key")]
pub struct InvalidKey;

