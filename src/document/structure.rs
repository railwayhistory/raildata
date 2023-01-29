
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{FullStore, StoreLoader, XrefsBuilder, XrefsStore};
use crate::types::{
    EventDate, Key, LanguageCode, LanguageText, List, LocalText, Marked
};
use super::{DocumentLink, SourceLink, StructureLink};
use super::common::{Common, Progress};


//------------ Link ----------------------------------------------------------

pub use super::combined::StructureLink as Link;


//------------ Document ------------------------------------------------------

pub use super::combined::StructureDocument as Document;

impl<'a> Document<'a> {
    pub fn json(self, _store: &FullStore) -> String {
        self.data().common.json(|json| {
            json.member_str("type", "structure");
        })
    }
}


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub link: StructureLink,
    pub common: Common,
    pub subtype: Marked<Subtype>,
    pub events: EventList,
}

impl Data {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    /// Returns the name for the given language.
    pub fn name(&self, lang: LanguageCode) -> &str {
        for event in self.events.iter().rev() {
            if let Some(name) = event.name.as_ref() {
                if let Some(name) = name.for_language(lang) {
                    return name
                }
            }
        }
        self.key().as_str()
    }
}

impl Data {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        link: DocumentLink,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take("subtype", context, report);
        let events = doc.take("events", context, report);
        doc.exhausted(report)?;
        Ok(Data {
            link: link.into(),
            common: common?,
            subtype: subtype?,
            events: events?,
        })
    }

    pub fn xrefs(
        &self, 
        _builder: &mut XrefsBuilder,
        _store: &crate::store::DataStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        Ok(())
    }

    pub fn catalogue(
        &self,
        builder: &mut CatalogueBuilder,
        _store: &FullStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        let mut names = HashSet::new();
        for event in &self.events {
            if let Some(some) = event.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            builder.insert_name(name.into(), self.link.into())
        }
        Ok(())
    }
}


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Xrefs;


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta;

impl Meta {
    pub fn generate(
        _data: &Data, _store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        Ok(Meta)
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Bridge: "bridge" }
        { Tunnel: "tunnel" }
    }
}


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    // Meta attributes
    pub date: EventDate,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
    pub note: Option<LanguageText>,

    pub length: Option<Marked<f64>>,
    pub name: Option<LocalText>,
}

impl FromYaml<StoreLoader> for Event {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let note = value.take_opt("note", context, report);
        let length = value.take_opt("length", context, report);
        let name = value.take_opt("name", context, report);
        value.exhausted(report)?;
        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            note: note?,
            length: length?,
            name: name?,
        })
    }
}

