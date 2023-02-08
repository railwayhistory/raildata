use httools::json::{JsonObject, JsonValue};
use crate::document::source::{Data, Document, Link};
use crate::http::json::StateBuildJson;
use crate::http::state::State;

impl<'a> Document<'a> {
    pub fn json(self, json: &mut JsonObject, state: &State) {
        self.data().common.json(json, state);
        json.string("type", self.doctype());
        json.value("subtype", |json| self.data().subtype.json(json, state));
        json.value("data", |json| self.data().json(json, state));
    }
}

impl StateBuildJson for Link {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            json.string("key", self.document(state.store()).key())
        })
    }
}

impl Data {
    fn json(&self, json: &mut JsonValue, state: &State) {
        json.object(|json| {
            if !self.author.is_empty() {
                json.value("author", |json| self.author.json(json, state));
            }
            if let Some(collection) = self.collection.as_ref() {
                json.value("collection", |json| collection.json(json, state));
            }
            self.date.json(json);
            if let Some(designation) = self.designation.as_ref() {
                json.string("designation", designation);
            }
            if !self.digital.is_empty() {
                json.value("digital", |json| self.digital.json(json, state));
            }
            if let Some(edition) = self.edition.as_ref() {
                json.string("edition", edition);
            }
            if !self.editor.is_empty() {
                json.value("editor", |json| self.editor.json(json, state));
            }
            if let Some(isbn) = self.isbn.as_ref() {
                json.string("isbn", isbn.as_str());
            }
            if let Some(number) = self.number.as_ref() {
                json.string("number", number)
            }
            if !self.organization.is_empty() {
                json.value("organization", |json| {
                    self.organization.json(json, state);
                })
            }
            if let Some(pages) = self.pages.as_ref() {
                json.string("pages", pages);
            }
            if !self.publisher.is_empty() {
                json.value("publisher", |json| {
                    self.publisher.json(json, state)
                })
            }
            if let Some(revision) = self.revision.as_ref() {
                json.string("revision", revision);
            }
            if let Some(short_title) = self.short_title.as_ref() {
                json.string("shortTitle", short_title);
            }
            if let Some(title) = self.title.as_ref() {
                json.string("title", title);
            }
            if let Some(url) = self.url.as_ref() {
                json.string("url", url);
            }
            if let Some(volume) = self.volume.as_ref() {
                json.string("volume", volume);
            }
            if !self.also.is_empty() {
                json.value("also", |json| self.also.json(json, state));
            }
            if let Some(attribution) = self.attribution.as_ref() {
                json.string("attribution", attribution);
            }
            if !self.crossref.is_empty() {
                json.value("crossref", |json| self.crossref.json(json, state))
            }
            if let Some(note) = self.note.as_ref() {
                json.value("note", |json| note.json(json, state))
            }
            if !self.regards.is_empty() {
                json.value("regards", |json| self.regards.json(json, state))
            }
        })
    }
}

