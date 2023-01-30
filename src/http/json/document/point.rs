use httools::json::JsonObject;
use crate::document::point::{Document};
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        self.data().common.json(json, state);
        json.string("type", self.doctype());
    }
}

