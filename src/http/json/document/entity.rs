use httools::json::JsonObject;
use crate::document::entity::{Data, Document};
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        self.data().common.json(json, state);
        json.string("type", self.doctype());
        json.object("data", |json| self.data().json(json, state));
    }
}

impl Data {
    pub fn json(&self, json: &mut JsonObject, _state: &State) {
        json.string("subtype", &self.subtype);
    }
}

