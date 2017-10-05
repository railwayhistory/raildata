use std::ops;
use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::{MarkedMapping, Value};
use super::common::Common;
use super::links::SourceLink;
use super::types::{EventDate, Float, LanguageText, List, LocalText};


//------------ Structure -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Structure {
    common: Common,
    subtype: Subtype,
    events: List<Event>,
}

impl Structure {
    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn events(&self) -> &List<Event> {
        &self.events
    }
}

impl Structure {
    pub fn construct<C>(common: Common, mut doc: MarkedMapping,
                        context: &mut C) -> Result<Self, Failed>
                     where C: Context {
        let subtype = doc.take("subtype", context);
        let events = doc.take("events", context);
        doc.exhausted(context)?;
        Ok(Structure { common,
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
    document: List<SourceLink>,
    source: List<SourceLink>,
    note: Option<LanguageText>,

    length: Option<Float>,
    name: Option<LocalText>,
}

impl Event {
    pub fn date(&self) -> &EventDate { &self.date }
    pub fn document(&self) -> &List<SourceLink> { &self.document }
    pub fn source(&self) -> &List<SourceLink> { &self.source }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }

    pub fn length(&self) -> Option<Float> { self.length }
    pub fn name(&self) -> Option<&LocalText> { self.name.as_ref() }
}

impl Constructable for Event {
    fn construct<C: Context>(value: Value, context: &mut C)
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
