
use std::{fmt, ops, str};
use std::str::FromStr;
use std::collections::HashMap;
use std::hash::Hash;
use ::load::construct::{Context, Constructable, Failed};
use ::load::yaml::Value;
use super::marked::Marked;
use super::simple::Text;


//------------ CountryCode ---------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    pub const DE: Self = CountryCode(*b"de");
}

impl ops::Deref for CountryCode {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl Constructable for Marked<CountryCode> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        Text::construct(value, context)?
             .try_map(|text| CountryCode::from_str(&text))
             .map_err(|err| { context.push_error(err); Failed })
    }
}

impl FromStr for CountryCode {
    type Err = CountryCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err(CountryCodeError(s.into()))
        }
        else {
            Ok(CountryCode([to_code_byte(s, 0)?,
                            to_code_byte(s, 1)?]))
        }
    }
}


//------------ LanguageCode --------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LanguageCode([u8; 3]);

impl ops::Deref for LanguageCode {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl Constructable for Marked<LanguageCode> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        Text::construct(value, context)?
             .try_map(|text| LanguageCode::from_str(&text))
             .map_err(|err| { context.push_error(err); Failed })
    }
}

impl FromStr for LanguageCode {
    type Err = LanguageCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 3 {
            Err(LanguageCodeError(s.into()))
        }
        else {
            Ok(LanguageCode([to_code_byte(s, 0)?,
                             to_code_byte(s, 1)?,
                             to_code_byte(s, 2)?]))
        }
    }
}


//------------ LocalCode -----------------------------------------------------

/// Either a county or a language code.
//
//  Internal coding: if the last byte is 0u8, we have a country code,
//  otherwise a language code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalCode([u8; 3]);

impl ops::Deref for LocalCode {
    type Target = str;

    fn deref(&self) -> &str {
        if self.0[2] == 0 {
            unsafe { str::from_utf8_unchecked(&self.0[..2]) }
        }
        else {
            unsafe { str::from_utf8_unchecked(&self.0) }
        }
    }
}

impl From<CountryCode> for LocalCode {
    fn from(code: CountryCode) -> LocalCode {
        LocalCode([code.0[0], code.0[1], 0])
    }
}

impl From<LanguageCode> for LocalCode {
    fn from(code: LanguageCode) -> LocalCode {
        LocalCode(code.0)
    }
}

impl Constructable for Marked<LocalCode> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        Text::construct(value, context)?
             .try_map(|text| LocalCode::from_str(&text))
             .map_err(|err| { context.push_error(err); Failed })
    }
}

impl FromStr for LocalCode {
    type Err = LocalCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 2 {
            Ok(LocalCode([to_code_byte(s, 0)?,
                          to_code_byte(s, 1)?,
                          0]))
        }
        else if s.len() == 3 {
            Ok(LocalCode([to_code_byte(s, 0)?,
                          to_code_byte(s, 1)?,
                          to_code_byte(s, 2)?]))
        }
        else {
            Err(LocalCodeError(s.into()))
        }
    }
}


//------------ CodedText and friends -----------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodedText<C: Hash + Eq>(CTInner<C>);

#[derive(Clone, Debug, Eq, PartialEq)]
enum CTInner<C: Hash + Eq> {
    Plain(Text),
    Map(HashMap<Marked<C>, Text>),
}

impl<C: Hash + Eq> CodedText<C> {
    pub fn from_text(text: Text) -> Self {
        CodedText(CTInner::Plain(text))
    }

    pub fn new_map() -> Self {
        CodedText(CTInner::Map(HashMap::new()))
    }

    pub fn insert(&mut self, key: Marked<C>, value: Text) {
        if let CTInner::Plain(_) = self.0 {
            self.0 = CTInner::Map(HashMap::new())
        }
        if let CTInner::Map(ref mut map) = self.0 {
            map.insert(key, value);
        }
        else {
            unreachable!()
        }
    }

    // XXX TODO Read access ...
}

impl<C: Hash + Eq + FromStr> Constructable for CodedText<C>
     where <C as FromStr>::Err: fmt::Display + fmt::Debug + 'static + Send {
    fn construct<Ctx: Context>(value: Value, context: &mut Ctx)
                               -> Result<Self, Failed> {
        match value.try_into_mapping() {
            Ok(mut value) => {
                let mut res = Self::new_map();
                let mut failed = value.check(context).is_err();
                for (key, value) in value {
                    let key = key.try_map(|s| C::from_str(&s));
                    let value = value.into_string(context);
                    if let Err(err) = key {
                        context.push_error(err);
                        failed = true;
                    }
                    else if value.is_err() {
                        failed = true;
                    }
                    else if !failed {
                        res.insert(key.unwrap(), value.unwrap());
                    }
                }
                if failed {
                    Err(Failed)
                }
                else {
                    Ok(res)
                }
            }
            Err(value) => {
                Text::construct(value, context).map(Self::from_text)
            }
        }
    }
}



pub type LocalText = CodedText<LocalCode>;
pub type LanguageText = CodedText<LanguageCode>;


//------------ CountryCodeError ----------------------------------------------

#[derive(Clone, Debug)]
pub struct CountryCodeError(String);

impl From<String> for CountryCodeError {
    fn from(s: String) -> Self {
        CountryCodeError(s)
    }
}

impl fmt::Display for CountryCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid country code '{}'", self.0)
    }
}


//------------ LanguageCodeError ---------------------------------------------

#[derive(Clone, Debug)]
pub struct LanguageCodeError(String);

impl From<String> for LanguageCodeError {
    fn from(s: String) -> Self {
        LanguageCodeError(s)
    }
}

impl fmt::Display for LanguageCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid language code '{}'", self.0)
    }
}


//------------ LocalCodeError ------------------------------------------------

#[derive(Clone, Debug)]
pub struct LocalCodeError(String);

impl From<String> for LocalCodeError {
    fn from(s: String) -> Self {
        LocalCodeError(s)
    }
}

impl fmt::Display for LocalCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid country or language code '{}'", self.0)
    }
}


//------------ Internal Helpers ----------------------------------------------

fn to_code_byte(s: &str, idx: usize) -> Result<u8, String> {
    let c = s.as_bytes()[idx];
    if c < b'A' || c > b'z' {
        Err(s.into())
    }
    else if c <= b'Z' {
        Ok(c)
    }
    else if c >= b'a' {
        Ok(c - (b'a' - b'A'))
    }
    else {
        Err(s.into())
    }
}

