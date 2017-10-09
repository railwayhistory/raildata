//! The attributes common to all documents.
//! 

use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::Value;
use ::load::path::Path;
use ::load::yaml::Mapping;
use super::links::{OrganizationLink, SourceLink};
use super::types::{Date, EventDate, Key, LanguageText, List, Location, Marked};


//------------ Common --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Common {
    key: Key,
    progress: Marked<Progress>,
    path: Path,
    location: Location,
}

impl Common {
    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn progress(&self) -> Progress {
        self.progress.to()
    }

    pub fn location(&self) -> (Path, Location) {
        (self.path.clone(), self.location)
    }
}

impl Common {
    pub fn construct<C>(key: &Key, doc: &mut Marked<Mapping>, path: Path,
                        context: &mut C) -> Result<Self, Failed>
                     where C: Context {
        Ok(Common {
            key: key.clone(),
            progress: doc.take_default("progress", context)?,
            path,
            location: doc.location()
        })
    }
}


//------------ Progress ------------------------------------------------------

data_enum! {
    pub enum Progress {
        { Stub: "stub" }
        { InProgress: "in-progress" }
        { Complete: "complete" }

        default InProgress
    }
}


//------------ Alternative ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct Alternative {
    date: EventDate,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
}

impl Alternative {
    pub fn date(&self) -> &EventDate { &self.date }
    pub fn document(&self) -> &List<Marked<SourceLink>> { &self.document }
    pub fn source(&self) -> &List<Marked<SourceLink>> { &self.source }
}

impl Constructable for Alternative {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let date = value.take("date", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        value.exhausted(context)?;
        Ok(Alternative {
            date: date?,
            document: document?,
            source: source?,
        })
    }
}


//------------ Basis ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Basis {
    date: Option<List<Marked<Date>>>,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    contract: Option<Contract>,
    treaty: Option<Contract>,
    note: Option<LanguageText>,
}

impl Basis {
    pub fn date(&self) -> Option<&List<Marked<Date>>> { self.date.as_ref() }
    pub fn document(&self) -> &List<Marked<SourceLink>> { &self.document }
    pub fn source(&self) -> &List<Marked<SourceLink>> { &self.source }
    pub fn contract(&self) -> Option<&Contract> { self.contract.as_ref() }
    pub fn treaty(&self) -> Option<&Contract> { self.treaty.as_ref() }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }
}

impl Constructable for Basis {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let date = value.take_opt("date", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        let contract = value.take_opt("contract", context);
        let treaty = value.take_opt("treaty", context);
        let note = value.take_opt("note", context);
        value.exhausted(context)?;
        Ok(Basis {
            date: date?,
            document: document?,
            source: source?,
            contract: contract?,
            treaty: treaty?,
            note: note?,
        })
    }
}


//------------ Contract ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Contract {
    parties: List<Marked<OrganizationLink>>,
}

impl Contract {
    pub fn parties(&self) -> &List<Marked<OrganizationLink>> { &self.parties }
}

impl Constructable for Contract {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let parties = value.take("parties", context);
        value.exhausted(context)?;
        Ok(Contract { parties: parties? })
    }
}

