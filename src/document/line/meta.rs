
use serde::{Deserialize, Serialize};
use crate::store::StoreEnricher;
use super::Data;


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
struct OrgList {
    items: (EventDate, Section, OrganizationLink),
}

