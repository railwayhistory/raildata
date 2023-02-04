use httools::json::{JsonObject, JsonValue};
use crate::document::line::{
    Concession, Current, CurrentValue, Data, Document, Event, Gauge, Link,
    Points, Properties, Record, RecordList, Section,
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

impl Data {
    pub fn json(&self, json: &mut JsonObject, state: &State) {
        // label
        if !self.label.is_empty() {
            json.array("label", |json| {
                for item in self.label.iter() {
                    json.string(item);
                }
            })
        }

        // note
        if let Some(note) = self.note.as_ref() {
            json.value("note", |json| note.json(json, state))
        }

        // current
        json.object("current", |json| self.current.json(json, state));

        // events
        if !self.events.is_empty() {
            json.value("events", |json| self.events.json(json, state));
        }

        // records
        if !self.records.is_empty() {
            json.value("records", |json| self.records.json(json, state));
        }

        // points
        json.value("points", |json| self.points.json(json, state));
    }
}

impl Points {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.array(|json| {
            self.iter().for_each(|item| {
                json.value(|json| item.points_json(json, state))
            });
        })
    }
}

impl Current {
    fn json(&self, json: &mut JsonObject, state: &State) {
        if !self.category.is_empty() {
            json.value("category", |json| self.category.json(json, state))
        }
        if !self.electrified.is_empty() {
            json.value(
                "electrified", |json| self.electrified.json(json, state)
            )
        }
        if !self.gauge.is_empty() {
            json.value("gauge", |json| self.gauge.json(json, state))
        }
        if !self.goods.is_empty() {
            json.value("goods", |json| self.goods.json(json, state))
        }
        if !self.jurisdiction.is_empty() {
            json.value(
                "jurisdiction", |json| self.jurisdiction.json(json, state)
            )
        }
        if !self.name.is_empty() {
            json.value("name", |json| self.name.json(json, state))
        }
        if !self.operator.is_empty() {
            json.value("operator", |json| self.operator.json(json, state))
        }
        if !self.owner.is_empty() {
            json.value("owner", |json| self.owner.json(json, state))
        }
        if !self.passenger.is_empty() {
            json.value("passenger", |json| self.passenger.json(json, state))
        }
        if !self.rails.is_empty() {
            json.value("rails", |json| self.rails.json(json, state))
        }
        if !self.region.is_empty() {
            json.value("region", |json| self.region.json(json, state))
        }
        if !self.reused.is_empty() {
            json.value("reused", |json| self.reused.json(json, state))
        }
        if !self.tracks.is_empty() {
            json.value("tracks", |json| self.tracks.json(json, state))
        }
        if !self.de_vzg.is_empty() {
            json.value("de.VzG", |json| self.de_vzg.json(json, state))
        }
        if !self.fr_rfn.is_empty() {
            json.value("fr.RFN", |json| self.fr_rfn.json(json, state))
        }
        if !self.source.is_empty() {
            json.value("source", |json| self.source.json(json, state))
        }
        if let Some(note) = self.note.as_ref() {
            json.value("note", |json| note.json(json, state))
        }
    }
}

impl<T> CurrentValue<T> {
    pub fn json( &self, json: &mut JsonValue, state: &State)
    where T: StateBuildJson {
        json.array(|json| {
            self.iter().for_each(|item| {
                json.object(|json| {
                    json.raw("start", item.0.start_idx);
                    json.raw("end", item.0.end_idx);
                    json.value(
                        "value", |json| item.1.json(json, state)
                    );
                })
            })
        })
    }
}

impl StateBuildJson for Event {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            // date
            self.date.json(json);

            // sections
            if !self.sections.is_empty() {
                json.value("sections", |json| self.sections.json(json, state))
            }

            // document
            if !self.document.is_empty() {
                json.value("document", |json| self.document.json(json, state))
            }

            // source
            if !self.source.is_empty() {
                json.value("source", |json| self.source.json(json, state))
            }

            // alternative
            if !self.alternative.is_empty() {
                json.value(
                    "alternative", |json| self.alternative.json(json, state)
                )
            }

            // basis
            if !self.basis.is_empty() {
                json.value("basis", |json| self.basis.json(json, state))
            }

            // note
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state))
            }

            // concession
            if let Some(concession) = self.concession.as_ref() {
                json.value("note", |json| concession.json(json, state))
            }

            // agreement
            if let Some(agreement) = self.agreement.as_ref() {
                json.value("agreement", |json| agreement.json(json, state))
            }

            self.properties.json(json, state)
        })
    }
}

impl RecordList {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.array(|json| {
            self.documents().for_each(|(source, records)| {
                json.object(|json| {
                    json.value("document", |json| source.json(json, state));
                    json.value("records", |json| records.json(json, state));
                })
            })
        })
    }
}

impl StateBuildJson for Record {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            json.value("sections", |json| self.sections.json(json, state));
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state));
            }
            self.properties.json(json, state);
        })
    }
}

impl Properties {
    fn json(&self, json: &mut JsonObject, state: &State) {
        if let Some(category) = self.category.as_ref() {
            json.value("category", |json| category.json(json, state));
        }
        if let Some(electrified) = self.electrified.as_ref() {
            json.value("electrified", |json| electrified.json(json, state));
        }
        if let Some(gauge) = self.gauge.as_ref() {
            json.value("gauge", |json| gauge.json(json, state));
        }
        if let Some(name) = self.name.as_ref() {
            json.value("name", |json| name.json(json, state));
        }
        if let Some(rails) = self.rails.as_ref() {
            json.value("rails", |json| rails.json(json, state));
        }
        if let Some(reused) = self.reused.as_ref() {
            json.value("reused", |json| reused.json(json, state));
        }
        if let Some(status) = self.status.as_ref() {
            json.value("status", |json| status.json(json, state));
        }
        if let Some(tracks) = self.tracks.as_ref() {
            json.value("tracks", |json| tracks.json(json, state));
        }

        if let Some(goods) = self.goods.as_ref() {
            json.value("goods", |json| goods.json(json, state));
        }
        if let Some(passenger) = self.passenger.as_ref() {
            json.value("passenger", |json| passenger.json(json, state));
        }

        if let Some(constructor) = self.constructor.as_ref() {
            json.value("constructor", |json| constructor.json(json, state));
        }
        if let Some(operator) = self.operator.as_ref() {
            json.value("operator", |json| operator.json(json, state));
        }
        if let Some(owner) = self.owner.as_ref() {
            json.value("owner", |json| owner.json(json, state));
        }
        if let Some(jurisdiction) = self.jurisdiction.as_ref() {
            json.value("jurisdiction", |json| jurisdiction.json(json, state));
        }

        if let Some(region) = self.region.as_ref() {
            json.value("region", |json| region.json(json, state));
        }

        if let Some(de_vzg) = self.de_vzg.as_ref() {
            json.value("de.VzG", |json| de_vzg.json(json, state));
        }
        if let Some(fr_rfn) = self.fr_rfn.as_ref() {
            json.value("fr.RFN", |json| fr_rfn.json(json, state));
        }
    }
}

impl StateBuildJson for Section {
    fn json(&self, json: &mut JsonValue, _state: &State) {
        json.object(|json| {
            json.raw("start", self.start_idx);
            json.raw("end", self.end_idx);
        })
    }
}

impl Concession {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            if !self.by.is_empty() {
                json.value("by", |json| self.by.json(json, state))
            }
            if !self.to.is_empty() {
                json.value("for", |json| self.to.json(json, state))
            }
            if !self.rights.is_empty() {
                json.value("rights", |json| self.rights.json(json, state))
            }
            if let Some(until) = self.until.as_ref() {
                json.value("until", |json| until.json(json))
            }
        })
    }
}

impl StateBuildJson for Gauge {
    fn json(&self, json: &mut JsonValue, _state: &State) {
        json.raw(self)
    }
}


