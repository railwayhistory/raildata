
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::types::{EventDate, Key, LanguageText, List, LocalText, Marked};
use super::SourceLink;
use super::common::{Common, Progress};
use super::store::{LoadStore, Stored};


//------------ Structure -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Structure {
    common: Common,
    subtype: Marked<Subtype>,
    events: EventList,
}

impl Structure {
    pub fn common(&self) -> &Common {
        &self.common
    }

    pub fn key(&self) -> &Key {
        self.common().key()
    }

    pub fn progress(&self) -> Progress {
        self.common().progress()
    }

    pub fn origin(&self) -> &Origin {
        &self.common().origin()
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype.into_value()
    }
}

impl<'a> Stored<'a, Structure> {
    pub fn events(&self) -> Stored<'a, EventList> {
        self.map(|item| &item.events)
    }
}

impl Structure {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &mut LoadStore,
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

    pub fn verify(&self, _report: &mut StageReporter) {
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

#[derive(Clone, Debug)]
pub struct Event {
    // Meta attributes
    date: EventDate,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    note: Option<LanguageText>,

    length: Option<Marked<f64>>,
    name: Option<LocalText>,
}

impl<'a> Stored<'a, Event> {
    pub fn date(&self) -> &EventDate {
        &self.access().date
    }

    pub fn document(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.document)
    }

    pub fn source(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.source)
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }

    pub fn length(&self) -> Option<f64> {
        self.access().length.map(Marked::into_value)
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.access().name.as_ref()
    }
}

impl FromYaml<LoadStore> for Event {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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

