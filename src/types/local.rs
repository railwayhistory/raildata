//! Types for holding localized data.

use std::{fmt, ops, str};
use std::str::FromStr;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::load::yaml::{FromYaml, Value};
use crate::load::report::{Failed, Message, PathReporter};
use super::marked::Marked;


//------------ CountryCode ---------------------------------------------------

#[derive(
    Clone, Copy, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    pub const AT: Self = CountryCode(*b"AT");
    pub const BE: Self = CountryCode(*b"BE");
    pub const CH: Self = CountryCode(*b"CH");
    pub const DD: Self = CountryCode(*b"DD");
    pub const DE: Self = CountryCode(*b"DE");
    pub const DK: Self = CountryCode(*b"DK");
    pub const FR: Self = CountryCode(*b"FR");
    pub const GB: Self = CountryCode(*b"GB");
    pub const LT: Self = CountryCode(*b"LT");
    pub const LU: Self = CountryCode(*b"LU");
    pub const NL: Self = CountryCode(*b"NL");
    pub const NO: Self = CountryCode(*b"NO");
    pub const PL: Self = CountryCode(*b"PL");
    pub const RU: Self = CountryCode(*b"RU");
    pub const SE: Self = CountryCode(*b"SE");
    pub const INVALID: Self = CountryCode(*b"XX");
}

impl CountryCode {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
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

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CountryCode('{}')", self.as_str())
    }
}


//------------ LanguageCode --------------------------------------------------

#[derive(
    Clone, Copy, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct LanguageCode([u8; 3]);

impl LanguageCode {
    pub const CES: Self = LanguageCode(*b"CES");
    pub const DAN: Self = LanguageCode(*b"DAN");
    pub const DEU: Self = LanguageCode(*b"DEU");
    pub const ENG: Self = LanguageCode(*b"ENG");
    pub const FRA: Self = LanguageCode(*b"FRA");
    pub const LAV: Self = LanguageCode(*b"LAV");
    pub const NOB: Self = LanguageCode(*b"NOB");
    pub const NLD: Self = LanguageCode(*b"NLD");
    pub const NNO: Self = LanguageCode(*b"NNO");
    pub const POL: Self = LanguageCode(*b"POL");
    pub const RUS: Self = LanguageCode(*b"RUS");
    pub const SWE: Self = LanguageCode(*b"SWE");
}

impl LanguageCode {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl ops::Deref for LanguageCode {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for LanguageCode {
    fn as_ref(&self) -> &str {
        self.as_str()
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

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LanguageCode('{}')", self.as_str())
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

impl LocalCode {
    pub fn try_into_language(self) -> Result<LanguageCode, CountryCode> {
        if self.0[2] == 0 {
            Err(CountryCode([self.0[0], self.0[1]]))
        }
        else {
            Ok(LanguageCode(self.0))
        }
    }

    pub fn as_str(&self) -> &str {
        if self.0[2] == 0 {
            unsafe { str::from_utf8_unchecked(&self.0[..2]) }
        }
        else {
            unsafe { str::from_utf8_unchecked(&self.0) }
        }
    }
}

impl ops::Deref for LocalCode {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
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
    pub fn first(&self) -> &str {
        match self.0 {
            CTInner::Plain(ref inner) => inner.as_str(),
            CTInner::Map(ref inner) => {
                inner.first().unwrap().1.as_str()
            }
        }
    }

    pub fn iter(&self) -> CodedTextIter<C> {
        CodedTextIter {
            text: self,
            pos: 0
        }
    }
}

impl<C: Ord + From<CountryCode>> CodedText<C> {
    pub fn for_jurisdiction(
        &self, jurisdiction: Option<CountryCode>
    ) -> Option<&str> {
        let jurisdiction = match jurisdiction {
            Some(jurisdiction) => C::from(jurisdiction),
            None => return Some(self.first())
        };
        match self.0 {
            CTInner::Plain(_) => None,
            CTInner::Map(ref inner) => {
                for &(ref code, ref text) in inner.iter() {
                    if *code.as_value() == jurisdiction {
                        return Some(text.as_str());
                    }
                }
                None
            }
        }
    }
}

impl<C: Ord + From<LanguageCode>> CodedText<C> {
    pub fn for_language(
        &self, language: LanguageCode
    ) -> Option<&str> {
        let language = C::from(language);
        match self.0 {
            CTInner::Plain(ref inner) => Some(inner.as_ref()),
            CTInner::Map(ref inner) => {
                for &(ref code, ref text) in inner.iter() {
                    if *code.as_value() == language {
                        return Some(text.as_str());
                    }
                }
                None
            }
        }
    }
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
                for (key, value) in value.into_iter() {
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

impl<'a, C: Ord> IntoIterator for &'a CodedText<C> {
    type Item = (Option<&'a Marked<C>>, &'a Marked<String>);
    type IntoIter = CodedTextIter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub type LocalText = CodedText<LocalCode>;
pub type LanguageText = CodedText<LanguageCode>;


//------------ CodedTextIter -------------------------------------------------

pub struct CodedTextIter<'a, C: Ord> {
    text: &'a CodedText<C>,
    pos: usize
}

impl<'a, C: Ord> Iterator for CodedTextIter<'a, C> {
    type Item = (Option<&'a Marked<C>>, &'a Marked<String>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.text.0 {
            CTInner::Plain(ref inner) => {
                if self.pos == 0 {
                    self.pos = 1;
                    Some((None, &inner))
                }
                else {
                    None
                }
            }
            CTInner::Map(ref inner) => {
                if self.pos < inner.len() {
                    let item = &inner[self.pos];
                    self.pos += 1;
                    Some((Some(&item.0), &item.1))
                }
                else {
                    None
                }
            }
        }
    }
}


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

