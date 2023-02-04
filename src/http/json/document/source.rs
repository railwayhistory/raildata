use httools::json::{JsonObject, JsonValue};
use crate::document::source::{Document, Link};
use crate::http::json::StateBuildJson;
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        self.data().common.json(json, state);
        json.string("type", self.doctype());
    }
}

impl StateBuildJson for Link {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            json.string("key", self.document(state.store()).key())
        })
    }
}

