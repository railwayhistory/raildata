use derive_more::Display;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as _;
use crate::load::yaml::{FromYaml, Value};
use crate::load::report::{Failed, PathReporter};
use super::{IntoMarked, Marked};


//------------ Url -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Url(url::Url);

impl Url {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<C> FromYaml<C> for Marked<Url> {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let value = value.into_string(report)?;
        match url::Url::parse(value.as_ref()) {
            Ok(url) => Ok(Url(url).marked(value.location())),
            Err(err) => {
                report.error(UrlError(err).marked(value.location()));
                Err(Failed)
            }
        }
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        self.0.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        url::Url::parse(&s)
            .map(Url)
            .map_err(D::Error::custom)
    }
}


//------------ UrlError ------------------------------------------------------

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid URL, {}", _0)]
struct UrlError(url::ParseError);

