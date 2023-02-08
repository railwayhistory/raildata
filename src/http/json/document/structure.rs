use httools::json::{JsonObject, JsonValue};
use crate::document::structure::{Data, Document, Event, Link};
use crate::http::json::StateBuildJson;
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        self.data().common.json(json, state);
        json.string("type", self.doctype());
        json.object("data", |json| self.data().json(json, state));
    }
}

impl StateBuildJson for Link {
    fn json(&self, json: &mut JsonValue, state: &State) {
        let doc = self.document(state.store());
        json.object(|json| {
            json.string("key", doc.key());
        })
    }
}

impl Data {
    fn json(&self, json: &mut JsonObject, state: &State) {
        json.value("subtype", |json| self.subtype.json(json, state));
        if !self.events.is_empty() {
            json.value("events", |json| self.events.json(json, state));
        }
    }
}

impl StateBuildJson for Event {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            self.date.json(json);
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state))
            }
            if !self.source.is_empty() {
                json.value("source", |json| self.source.json(json, state))
            }
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state));
            }

            if let Some(length) = self.length {
                json.raw("length", format_args!("{:.1}", length));
            }
            if let Some(name) = self.name.as_ref() {
                json.value("name", |json| name.json(json, state));
            }
        })
    }
}

