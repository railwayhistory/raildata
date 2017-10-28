use std::fmt;
use url;
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::yaml::Value;
use super::Marked;

pub use url::Url;

impl Constructable for Marked<Url> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let value = value.into_string(context)?;
        match url::Url::parse(value.as_ref()) {
            Ok(url) => Ok(Marked::new(url, value.location())),
            Err(err) => {
                context.push_error((UrlError(err), value.location()));
                Err(Failed)
            }
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct UrlError(url::ParseError);

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invaldid URL, {}", self.0)
    }
}

