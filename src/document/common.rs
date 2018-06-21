//! Attributes and attribute types common to all documents.

use ::load::report::{Failed, Origin, PathReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::types::{Date, EventDate, Key, LanguageText, List, Marked};
use super::organization::OrganizationLink;
use super::source::SourceLink;
use super::store::{DocumentStoreBuilder, Stored};


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
        context: &mut DocumentStoreBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(Common {
            key: key,
            progress: doc.take_default("progress", context, report)?,
            origin: Origin::new(report.path().clone(), doc.location())
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

impl<'a> Stored<'a, Alternative> {
    pub fn date(&self) -> &'a EventDate {
        &self.access().date
    }

    pub fn document(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.document)
    }

    pub fn source(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.source)
    }
}

impl FromYaml<DocumentStoreBuilder> for Alternative {
    fn from_yaml(
        value: Value,
        context: &mut DocumentStoreBuilder,
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
    date: Option<List<Marked<Date>>>,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    contract: Option<Contract>,
    treaty: Option<Contract>,
    note: Option<LanguageText>,
}

impl<'a> Stored<'a, Basis> {
    pub fn date(&self) -> Option<&List<Marked<Date>>> {
        self.access().date.as_ref()
    }

    pub fn document(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.document)
    }

    pub fn source(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.source)
    }

    pub fn contract(&self) -> Option<Stored<'a, Contract>> {
        self.map_opt(|item| item.contract.as_ref())
    }

    pub fn treaty(&self) -> Option<Stored<'a, Contract>> {
        self.map_opt(|item| item.treaty.as_ref())
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }
}

impl FromYaml<DocumentStoreBuilder> for Basis {
    fn from_yaml(
        value: Value,
        context: &mut DocumentStoreBuilder,
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

impl<'a> Stored<'a, Contract> {
    pub fn parties(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.parties)
    }
}

impl FromYaml<DocumentStoreBuilder> for Contract {
    fn from_yaml(
        value: Value,
        context: &mut DocumentStoreBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let parties = value.take("parties", context, report);
        value.exhausted(report)?;
        Ok(Contract { parties: parties? })
    }
}

