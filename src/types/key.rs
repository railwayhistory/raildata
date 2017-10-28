
use std::{fmt, str};
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::yaml::Value;
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
    pub fn from_string(s: Marked<String>, _context: &mut ConstructContext)
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

impl Constructable for Marked<Key> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        Ok(value.into_string(context)?.map(Key))
    }
}

impl Constructable for Key {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        Marked::construct(value, context).map(Marked::into_value)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}


//------------ InvalidKey ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct InvalidKey;

impl fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid key")
    }
}

