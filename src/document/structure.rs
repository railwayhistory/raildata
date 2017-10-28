use std::ops;
use ::links::SourceLink;
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::yaml::{Mapping, Value};
use ::types::{EventDate, Key, LanguageText, List, LocalText, Marked};
use super::common::Common;


//------------ Structure -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Structure {
    common: Common,
    subtype: Marked<Subtype>,
    events: List<Event>,
}

impl Structure {
    pub fn subtype(&self) -> Subtype {
        self.subtype.to_value()
    }

    pub fn events(&self) -> &List<Event> {
        &self.events
    }
}

impl Structure {
    pub fn construct(key: Marked<Key>, mut doc: Marked<Mapping>,
                     context: &mut ConstructContext) -> Result<Self, Failed> {
        let common = Common::construct(key, &mut doc, context);
        let subtype = doc.take("subtype", context);
        let events = doc.take("events", context);
        doc.exhausted(context)?;
        Ok(Structure {
            common: common?,
            subtype: subtype?,
            events: events?,
        })
    }
}

impl ops::Deref for Structure {
    type Target = Common;

    fn deref(&self) -> &Common {
        &self.common
    }
}

impl ops::DerefMut for Structure {
    fn deref_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Bridge: "bridge" }
        { Tunnel: "tunnel" }
    }
}


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
    pub fn date(&self) -> &EventDate { &self.date }
    pub fn document(&self) -> &List<Marked<SourceLink>> { &self.document }
    pub fn source(&self) -> &List<Marked<SourceLink>> { &self.source }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }

    pub fn length(&self) -> Option<f64> {
        self.length.as_ref().map(Marked::to_value)
    }
    pub fn name(&self) -> Option<&LocalText> { self.name.as_ref() }
}

impl Constructable for Event {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let date = value.take("date", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        let note = value.take_opt("note", context);
        let length = value.take_opt("length", context);
        let name = value.take_opt("name", context);
        value.exhausted(context)?;
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

