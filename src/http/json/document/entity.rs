use httools::json::{JsonObject, JsonValue};
use crate::document::entity::{Data, Document, Event, Link, Property};
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
            json.string("key", doc.key())
        })
    }
}

impl Data {
    pub fn json(&self, json: &mut JsonObject, state: &State) {
        json.string("subtype", &self.subtype);
        json.value("events", |json| self.events.json(json, state));
    }
}

impl StateBuildJson for Event {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            // date
            self.date.json(json);

            // document
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state));
            }

            // source
            if !self.source.is_empty() {
                json.value("source", |json| self.source.json(json, state));
            }

            // basis
            if !self.basis.is_empty() {
                json.value("basis", |json| self.basis.json(json, state));
            }

            // note
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state))
            }

            // domicile
            if !self.domicile.is_empty() {
                json.value("domicile", |json| self.domicile.json(json, state))
            }

            // name
            if let Some(name) = self.name.as_ref() {
                json.value("name", |json| name.json(json, state));
            }

            // owner
            if let Some(owner) = self.owner.as_ref() {
                json.value("owner", |json| owner.json(json, state))
            }

            // property
            if let Some(property) = self.property.as_ref() {
                json.object("property", |json| property.json(json, state));
            }

            // short_name
            if let Some(short_name) = self.short_name.as_ref() {
                json.value(
                    "shortName", |json| short_name.json(json, state)
                )
            }

            // status
            if let Some(status) = self.status {
                json.string("status", status);
            }

            // successor
            if let Some(successor) = self.successor {
                json.value("successor", |json| {
                    successor.json(json, state)
                })
            }

            // superior
            if let Some(superior) = self.superior {
                json.value("superior", |json| superior.json(json, state))
            }
        })
    }
}

impl Property {
    pub fn json(&self, json: &mut JsonObject, state: &State) {
        json.string("role", self.role);
        if !self.region.is_empty() {
            json.value("region", |json| self.region.json(json, state));
        }
        if !self.constructor.is_empty() {
            json.value("constructor", |json| {
                self.constructor.json(json, state)
            })
        }
        if !self.operator.is_empty() {
            json.value("operator", |json| self.operator.json(json, state))
        }
        if !self.owner.is_empty() {
            json.value("owner", |json| self.owner.json(json, state))
        }
    }
}

