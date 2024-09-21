
use std::collections::HashSet;
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{
    DataStore, DocumentLink, FullStore, StoreLoader, XrefsBuilder, XrefsStore,
};
use crate::types::{
    EventDate, Key, LanguageText, List, LocalText, Marked, Set,
};
use super::source;
use super::common::{Common, Progress};


//------------ Link ----------------------------------------------------------

pub use super::combined::StructureLink as Link;


//------------ Document ------------------------------------------------------

pub use super::combined::StructureDocument as Document;

impl<'a> Document<'a> {
}


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Data {
    link: Link,
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

    pub fn link(&self) -> Link {
        self.link
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

#[derive(Clone, Debug, Default)]
pub struct Xrefs {
    source_regards: Set<source::Link>,
}

impl Xrefs {
    pub fn source_regards_mut(&mut self) -> &mut Set<source::Link> {
        &mut self.source_regards
    }

    pub fn finalize(&mut self, _store: &DataStore) {
    }
}


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug)]
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
        { Tunnel: "tunnel" }
        { Viaduct: "viaduct" }
    }
}


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Event {
    // Meta attributes
    pub date: EventDate,
    pub document: List<Marked<source::Link>>,
    pub source: List<Marked<source::Link>>,
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

