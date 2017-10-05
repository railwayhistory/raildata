
use std::fmt;
use ::load::construct::{Context, Constructable, Failed};
use ::load::yaml::Value;
use super::simple::Text;
use super::marked::{Location, Marked};


//------------ Key -----------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(Text);


impl Key {
    pub fn location(&self) -> Location {
        self.0.location()
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Key(Text::new(s, Location::default()))
    }
}

impl From<Marked<String>> for Key {
    fn from(s: Marked<String>) -> Self {
        Key(s)
    }
}

impl Constructable for Key {
    fn construct<C>(value: Value, context: &mut C) -> Result<Self, Failed>
                 where C: Context {
        Text::construct(value, context).map(Key)
    }
}


impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}
