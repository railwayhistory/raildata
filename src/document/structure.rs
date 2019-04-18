
use crate::library::{LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{EventDate, Key, LanguageText, List, LocalText, Marked};
use super::{SourceLink, StructureLink};
use super::common::{Common, Progress};

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

    pub fn events(&self) -> &EventList {
        &self.events
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

impl Event {
    pub fn date(&self) -> &EventDate {
        &self.date
    }

    pub fn document(&self) -> &List<Marked<SourceLink>> {
        &self.document
    }

    pub fn source(&self) -> &List<Marked<SourceLink>> {
        &self.source
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.note.as_ref()
    }

    pub fn length(&self) -> Option<f64> {
        self.length.map(Marked::into_value)
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.name.as_ref()
    }
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

