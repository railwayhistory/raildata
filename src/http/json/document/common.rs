use httools::json::JsonObject;
use crate::document::common::Common;
use crate::http::state::State;

impl Common {
    pub fn json(&self, json: &mut JsonObject, _state: &State) {
        json.string("key", &self.key);
        json.string("progress", &self.progress);
    }
}

