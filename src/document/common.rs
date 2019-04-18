//! Attributes and attribute types common to all documents.

use crate::library::LibraryBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{EventDate, Key, LanguageText, List, Marked, Set};
use super::{OrganizationLink, SourceLink};


//------------ Common --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Common {
    //--- Attributes
    pub key: Marked<Key>,
    pub progress: Marked<Progress>,
    pub origin: Origin,

    //--- Cross-links

    /// Sources that have `regards` entries for this document.
    pub sources: Set<SourceLink>,
}

impl Common {
    pub fn new(
        key: Marked<Key>,
        progress: Marked<Progress>,
        origin: Origin
    ) -> Self {
        Common {
            key,
            progress,
            origin,
            sources: Set::new(),
        }
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
            origin: Origin::new(report.path().clone(), doc.location()),
            sources: Set::new(),
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
    pub date: EventDate,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
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
    pub date: Option<EventDate>,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
    pub contract: Option<Contract>,
    pub treaty: Option<Contract>,
    pub note: Option<LanguageText>,
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
    pub parties: List<Marked<OrganizationLink>>,
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

