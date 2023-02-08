use httools::json::{JsonObject, JsonValue};
use crate::document::combined::{Document, Link};
use crate::http::json::StateBuildJson;
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        match self {
            Document::Entity(inner) => inner.json(json, state),
            Document::Line(inner) => inner.json(json, state),
            Document::Path(inner) => inner.json(json, state),
            Document::Point(inner) => inner.json(json, state),
            Document::Source(inner) => inner.json(json, state),
            Document::Structure(inner) => inner.json(json, state),
        }
    }
}

impl StateBuildJson for Link {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            json.string("key", self.document(state.store()).key())
        })
    }
}

