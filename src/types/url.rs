use url;
use ::load::yaml::{FromYaml, Value};
use ::load::report::{Failed, PathReporter};
use super::{IntoMarked, Marked};

pub use url::Url;

impl<C> FromYaml<C> for Marked<Url> {
    fn from_yaml(
        value: Value,
        _: &mut C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let value = value.into_string(report)?;
        match url::Url::parse(value.as_ref()) {
            Ok(url) => Ok(url.marked(value.location())),
            Err(err) => {
                report.error(UrlError(err).marked(value.location()));
                Err(Failed)
            }
        }
    }
}


#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid URL, {}", _0)]
struct UrlError(url::ParseError);

