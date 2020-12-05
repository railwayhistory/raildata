
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::library::{LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{
    EventDate, Key, LanguageCode, LanguageText, List, LocalText, Marked
};
use super::{SourceLink, StructureLink};
use super::common::{Common, Progress};

//------------ Structure -----------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Structure {
    pub common: Common,
    pub subtype: Marked<Subtype>,
    pub events: EventList,
}

impl Structure {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    pub fn name(&self, _lang: LanguageCode) -> &str {
        self.key().as_str()
    }
}

impl Structure {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take("subtype", context, report);
        let events = doc.take("events", context, report);
        doc.exhausted(report)?;
        Ok(Structure {
            common: common?,
            subtype: subtype?,
            events: events?,
        })
    }

    pub fn crosslink(
        &self,
        _link: StructureLink,
        _library: &LibraryMut,
        _report: &mut StageReporter
    ) {
    }

    /*
    pub fn verify(&self, _report: &mut StageReporter) {
    }
    */

    pub fn process_names<F: FnMut(String)>(&self, mut process: F) {
        let mut names = HashSet::new();
        for event in &self.events {
            if let Some(some) = event.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            process(name.into())
        }
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

impl FromYaml<LibraryBuilder> for Event {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
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

