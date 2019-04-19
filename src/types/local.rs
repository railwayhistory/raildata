//! Types for holding localized data.

use std::{ops, str};
use std::str::FromStr;
use ::load::yaml::{FromYaml, Value};
use ::load::report::{Failed, Message, PathReporter};
use super::marked::Marked;


//------------ CountryCode ---------------------------------------------------

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
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

impl<C> FromYaml<C> for Marked<CountryCode> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_string(report)?
             .try_map(|text| CountryCode::from_str(&text))
             .map_err(|err| { report.error(err); Failed })
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

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct LanguageCode([u8; 3]);

impl ops::Deref for LanguageCode {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<C> FromYaml<C> for Marked<LanguageCode> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_string(report)?
             .try_map(|text| LanguageCode::from_str(&text))
             .map_err(|err| { report.error(err); Failed })
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
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
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

impl<C> FromYaml<C> for Marked<LocalCode> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        value.into_string(report)?
             .try_map(|text| LocalCode::from_str(&text))
             .map_err(|err| { report.error(err); Failed })
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodedText<C: Ord>(CTInner<C>);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum CTInner<C: Ord> {
    Plain(Marked<String>),
    Map(Vec<(Marked<C>, Marked<String>)>),
}

impl<C: Ord> CodedText<C> {
    // XXX TODO Read access ...
}

impl<Ctx, C: Ord + FromStr> FromYaml<Ctx> for CodedText<C>
where <C as FromStr>::Err: Message {
    fn from_yaml(
        value: Value,
        _: &Ctx,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        match value.try_into_mapping() {
            Ok(mut value) => {
                let mut res = Vec::new();
                let mut failed = value.check(report).is_err();
                for (key, value) in value {
                    let key = key.try_map(|s| C::from_str(&s))
                                 .map_err(|err| { report.error(err); Failed });
                    let value = value.into_string(report);
                    match (key, value, failed) {
                        (Ok(key), Ok(value), false) => {
                            res.push((key, value));
                        }
                        _ => failed = true
                    }
                }
                if failed {
                    Err(Failed)
                }
                else {
                    use ::std::cmp::Ord;

                    res.sort_by(|left, right| left.0.cmp(&right.0));
                    Ok(CodedText(CTInner::Map(res)))
                }
            }
            Err(value) => {
                value.into_string(report).map(|res| {
                    CodedText(CTInner::Plain(res))
                })
            }
        }
    }
}


pub type LocalText = CodedText<LocalCode>;
pub type LanguageText = CodedText<LanguageCode>;


//------------ CountryCodeError ----------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="invalid country code '{}'", _0)]
pub struct CountryCodeError(String);

impl From<String> for CountryCodeError {
    fn from(s: String) -> Self {
        CountryCodeError(s)
    }
}


//------------ LanguageCodeError ---------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="invalid language code '{}'", _0)]
pub struct LanguageCodeError(String);

impl From<String> for LanguageCodeError {
    fn from(s: String) -> Self {
        LanguageCodeError(s)
    }
}


//------------ LocalCodeError ------------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="invalid country or language code '{}'", _0)]
pub struct LocalCodeError(String);

impl From<String> for LocalCodeError {
    fn from(s: String) -> Self {
        LocalCodeError(s)
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

