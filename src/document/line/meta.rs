
use crate::store::XrefsStore;
use crate::load::report::{Failed, PathReporter};
use super::data::Data;


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Meta {
}

impl Meta {
    pub fn generate(
        _data: &Data, _store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        Ok(Meta { })
    }
}

