use std::fmt;
use url;
use ::load::construct::{Context, Constructable, Failed};
use ::load::yaml::Value;
use super::Marked;

pub type Url = Marked<url::Url>;

impl Constructable for Url {
    fn construct<C>(value: Value, context: &mut C) -> Result<Self, Failed>
                 where C: Context {
        let value = value.into_string(context)?;
        match url::Url::parse(value.as_ref()) {
            Ok(url) => Ok(Url::new(url, value.location())),
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

