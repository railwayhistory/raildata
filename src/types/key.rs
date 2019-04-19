
use std::{borrow, fmt, ops, str};
use ::load::report::{Failed, PathReporter};
use ::load::yaml::{FromYaml, Value};
use super::marked::Marked;


//------------ Key -----------------------------------------------------------

#[derive(
    Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize
)]
pub struct Key(String);

impl Key {
    pub fn from_string(s: String) -> Result<Self, InvalidKey> {
        Ok(Key(s))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl Marked<Key> {
    pub fn from_string(s: Marked<String>, _report: &mut PathReporter)
                       -> Result<Self, Failed> {
        Ok(s.map(Key))
    }
}

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


impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}


//------------ InvalidKey ----------------------------------------------------

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid key")]
pub struct InvalidKey;

