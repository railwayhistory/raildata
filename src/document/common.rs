//! Attributes and attribute types common to all documents.

use crate::library::LibraryBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{EventDate, Key, LanguageText, List, Marked};
use super::{OrganizationLink, SourceLink};


//------------ Common --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Common {
    key: Marked<Key>,
    progress: Marked<Progress>,
    origin: Origin,
}

impl Common {
    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn progress(&self) -> Progress {
        self.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }
}

impl Common {
    pub fn new(
        key: Marked<Key>,
        progress: Marked<Progress>,
        origin: Origin
    ) -> Self {
        Common { key, progress, origin }
    }

    pub fn from_yaml(
        key: Marked<Key>,
        doc: &mut Mapping,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(Common {
            key: key,
            progress: doc.take_default("progress", context, report)?,
            origin: Origin::new(report.path().clone(), doc.location())
        })
    }
}


//------------ DocumentType --------------------------------------------------

data_enum! {
    pub enum DocumentType {
        { Line: "line" }
        { Organization: "organization" }
        { Path: "path" }
        { Point: "point" }
        { Source: "source" }
        { Structure: "structure" }
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
    pub fn date(&self) -> &EventDate {
        &self.date
    }

    pub fn document(&self) -> &List<Marked<SourceLink>> {
        &self.document
    }

    pub fn source(&self) -> &List<Marked<SourceLink>> {
        &self.source
    }
}

impl FromYaml<LibraryBuilder> for Alternative {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        value.exhausted(report)?;
        Ok(Alternative {
            date: date?,
            document: document?,
            source: source?
        })
    }
}


//------------ Basis ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Basis {
    date: Option<EventDate>,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    contract: Option<Contract>,
    treaty: Option<Contract>,
    note: Option<LanguageText>,
}

impl Basis {
    pub fn date(&self) -> Option<&EventDate> {
        self.date.as_ref()
    }

    pub fn document(&self) -> &List<Marked<SourceLink>> {
        &self.document
    }

    pub fn source(&self) -> &List<Marked<SourceLink>> {
        &self.source
    }

    pub fn contract(&self) -> Option<&Contract> {
        self.contract.as_ref()
    }

    pub fn treaty(&self) -> Option<&Contract> {
        self.treaty.as_ref()
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.note.as_ref()
    }
}

impl FromYaml<LibraryBuilder> for Basis {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take_opt("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let contract = value.take_opt("contract", context, report);
        let treaty = value.take_opt("treaty", context, report);
        let note = value.take_opt("note", context, report);
        value.exhausted(report)?;
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
    pub fn parties(&self) -> &List<Marked<OrganizationLink>> {
        &self.parties
    }
}

impl FromYaml<LibraryBuilder> for Contract {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let parties = value.take("parties", context, report);
        value.exhausted(report)?;
        Ok(Contract { parties: parties? })
    }
}

