
use std::{borrow, fmt};
use ::load::construct::{Context, Constructable, Failed};
use ::load::yaml::Value;
use super::marked::Marked;


//------------ Boolean -------------------------------------------------------

pub type Boolean = Marked<bool>;

impl Constructable for Boolean {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        value.into_boolean(context)
    }
}


//------------ Float ---------------------------------------------------------

pub type Float = Marked<f64>;

impl Constructable for Float {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        value.into_float(context)
    }
}


//------------ Text ----------------------------------------------------------

pub type Text = Marked<String>;

impl borrow::Borrow<str> for Text {
    fn borrow(&self) -> &str {
        self.as_value().borrow()
    }
}

impl Constructable for Text {
    fn construct<C>(value: Value, context: &mut C) -> Result<Self, Failed>
                 where C: Context {
        value.into_string(context)
    }
}


//------------ Uint8 ---------------------------------------------------------

pub type Uint8 = Marked<u8>;

impl From<Uint8> for u8 {
    fn from(x: Uint8) -> u8 {
        x.to()
    }
}

impl Constructable for Uint8 {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        value.into_integer(context)?
             .try_map(|int| {
                if int < 0 || int > ::std::u8::MAX as i64 {
                    Err(RangeError::new(0, ::std::u8::MAX as i64, int))
                }
                else {
                    Ok(int as u8)
                }
             })
            .map_err(|err| { context.push_error(err); Failed })
    }
}


//------------ EnumError -----------------------------------------------------

pub struct EnumError(String);

impl EnumError {
    pub fn new(variant: String) -> Self {
        EnumError(variant)
    }
}

impl fmt::Display for EnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid enum value '{}'", self.0)
    }
}


//------------ RangeError ----------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct RangeError {
    low: i64,
    hi: i64,
    is: i64
}

impl RangeError {
    pub fn new(low: i64, hi: i64, is: i64) -> Self {
        RangeError { low, hi, is }
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value {} is outside of range {} to {}",
               self.is, self.low, self.hi)
    }
}

