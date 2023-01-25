
use serde::{Deserialize, Serialize};
use crate::store::StoreEnricher;
use crate::document::combined::EntityLink;
use crate::types::EventDate;
use super::data::{Data, Section};


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
}

impl Meta {
    pub(super) fn generate(_data: &Data, _store: &StoreEnricher) -> Self {
        Meta {
        }
    }
}


//------------ OrgList -------------------------------------------------------

#[derive(Clone, Debug)]
#[allow(dead_code)] // XXX
struct OrgList {
    items: (EventDate, Section, EntityLink),
}

