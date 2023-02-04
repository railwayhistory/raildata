use httools::json::{JsonObject, JsonValue};
use crate::document::point::{
    Codes, Data, Document, Event, Link, Location, Properties, Record, Site,
};
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

impl Link {
    pub fn points_json(&self, json: &mut JsonValue, state: &State) {
        let doc = self.document(state.store());
        json.object(|json| {
            json.string("key", doc.key());
            json.raw("junction", doc.meta().junction);
        })
    }
}

impl Data {
    fn json(&self, json: &mut JsonObject, state: &State) {
        json.value("subtype", |json| self.subtype.json(json, state));
        if let Some(junction) = self.junction {
            json.raw("junction", junction)
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
            if !self.basis.is_empty() {
                json.value("basis", |json| self.basis.json(json, state));
            }
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state));
            }
            if let Some(split_from) = self.split_from.as_ref() {
                json.value("splitFrom", |json| split_from.json(json, state));
            }
            if let Some(merged) = self.merged.as_ref() {
                json.value("merged", |json| merged.json(json, state));
            }
            if let Some(connection) = self.connection.as_ref() {
                json.value("connection", |json| connection.json(json, state));
            }
            if let Some(site) = self.site.as_ref() {
                json.value("site", |json| site.json(json, state));
            }
            self.properties.json(json, state);
        });
    }
}

impl StateBuildJson for Record {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state))
            }
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state));
            }
            self.properties.json(json, state);
        })
    }
}

impl Properties {
    fn json(&self, json: &mut JsonObject, state: &State) {
        if let Some(status) = self.status {
            json.value("status", |json| status.json(json, state));
        }
        if let Some(name) = self.name.as_ref() {
            json.value("name", |json| name.json(json, state));
        }
        if let Some(name) = self.short_name.as_ref() {
            json.value("shortName", |json| name.json(json, state));
        }
        if let Some(name) = self.public_name.as_ref() {
            json.value("publicName", |json| name.json(json, state));
        }
        if let Some(name) = self.designation.as_ref() {
            json.value("designation", |json| name.json(json, state));
        }
        if let Some(name) = self.de_name16.as_ref() {
            json.value("de.name16", |json| name.json(json, state));
        }

        if let Some(category) = self.category.as_ref() {
            json.value("category", |json| category.json(json, state));
        }
        if let Some(rang) = self.de_rang.as_ref() {
            json.value("de.rang", |json| rang.json(json, state));
        }
        if let Some(superior) = self.superior.as_ref() {
            json.value("superior", |json| superior.json(json, state));
        }
        self.codes.json(json, state);
        if !self.location.is_empty() {
            json.value("location", |json| self.location.json(json, state));
        }

        if let Some(staff) = self.staff {
            json.value("staff", |json| staff.json(json, state));
        }
        if let Some(service) = self.service {
            json.value("service", |json| service.json(json, state));
        }
        if let Some(service) = self.passenger {
            json.value("passenger", |json| service.json(json, state));
        }
        if let Some(service) = self.luggage {
            json.value("luggage", |json| service.json(json, state));
        }
        if let Some(service) = self.express {
            json.value("express", |json| service.json(json, state));
        }
        if let Some(service) = self.goods {
            json.value("goods", |json| service.json(json, state));
        }
    }
}

impl Location {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.array(|json| {
            self.iter().for_each(|item| {
                json.object(|json| {
                    json.value("line", |json| item.0.json(json, state));
                    match item.1 {
                        Some(value) => json.string("location", value),
                        None => json.raw("location", "null"),
                    }
                })
            })
        })
    }
}

impl Site {
    fn json(&self, _json: &mut JsonValue, _state: &State) {
        unimplemented!()
    }
}

impl Codes {
    fn json(&self, _json: &mut JsonObject, _state: &State) {
        unimplemented!()
    }
}

