use httools::json::JsonObject;
use crate::document::combined::{Document};
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

