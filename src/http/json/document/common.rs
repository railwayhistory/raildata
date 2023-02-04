use httools::json::{JsonObject, JsonValue};
use crate::document::common::{Agreement, Alternative, Basis, Common, Contract};
use crate::http::json::StateBuildJson;
use crate::http::state::State;

impl Common {
    pub fn json(&self, json: &mut JsonObject, _state: &State) {
        json.string("key", &self.key);
        json.string("progress", &self.progress);
    }
}

impl StateBuildJson for Alternative {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            // date
            self.date.json(json);

            // document
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state))
            }

            // source
            if !self.source.is_empty() {
                json.value("source", |json| self.source.json(json, state))
            }
        })
    }
}

impl StateBuildJson for Basis {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            // date
            self.date.json(json);

            // document
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state))
            }

            // source
            if !self.source.is_empty() {
                json.value("source", |json| self.source.json(json, state))
            }

            // agreement
            if let Some(agreement) = self.agreement.as_ref() {
                json.value("agreement", |json| agreement.json(json, state))
            }

            // note
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state))
            }
        })
    }
}

impl Agreement {
    pub fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            json.string("type", self.agreement_type);
            json.value("parties", |json| self.parties.json(json, state));
        })
    }
}

impl Contract {
    pub fn json(&self, json: &mut JsonObject, state: &State) {
        json.value("parties", |json| self.parties.json(json, state))
    }
}

