use serde::{Deserialize, Serialize};
use crate::document::source;
use crate::store::DataStore;
use crate::types::Set;


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Xrefs {
    source_regards: Set<source::Link>,
}

impl Xrefs {
    pub fn source_regards_mut(&mut self) -> &mut Set<source::Link> {
        &mut self.source_regards
    }

    pub fn finalize(&mut self, _store: &DataStore) {
    }
}

