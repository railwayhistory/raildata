//! Attributes and attribute types common to all documents.

use derive_more::Display;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::StoreLoader;
use crate::types::{
    EventDate, IntoMarked, Key, LanguageText, List, Location, Marked,
};
use super::{entity, source};


//------------ Common --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Common {
    //--- Attributes
    pub key: Marked<Key>,
    pub progress: Marked<Progress>,
    pub origin: Origin,
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
        }
    }

    pub fn from_yaml(
        key: Marked<Key>,
        doc: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(Common {
            key: key,
            progress: doc.take_default("progress", context, report)?,
            origin: Origin::new(report.path().clone(), doc.location()),
        })
    }
}


//------------ DocumentType --------------------------------------------------

data_enum! {
    pub enum DocumentType {
        { Line: "line" }
        { Entity: "entity" }
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

impl Progress {
    pub fn is_stub(self) -> bool {
        matches!(self, Progress::Stub)
    }
}


//------------ Alternative ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct Alternative {
    pub date: EventDate,
    pub document: List<Marked<source::Link>>,
    pub source: List<Marked<source::Link>>,
}


impl FromYaml<StoreLoader> for Alternative {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
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
    pub date: EventDate,
    pub document: List<Marked<source::Link>>,
    pub source: List<Marked<source::Link>>,
    pub agreement: Option<Agreement>,
    pub note: Option<LanguageText>,
}

impl FromYaml<StoreLoader> for Basis {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take_default("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let agreement = value.take_opt("agreement", context, report);
        let contract: Result<Option<Contract>, _>
            = value.take_opt("contract", context, report);
        let treaty: Result<Option<Contract>, _>
            = value.take_opt("treaty", context, report);
        let note = value.take_opt("note", context, report);
        value.exhausted(report)?;

        let agreement = agreement?;
        let contract = contract?;
        let treaty = treaty?;

        let agreement = if let Some(agreement) = agreement {
            if let Some(contract) = contract {
                report.error(MultipleAgreements.marked(contract.pos));
                return Err(Failed)
            }
            if let Some(treaty) = treaty {
                report.error(MultipleAgreements.marked(treaty.pos));
                return Err(Failed)
            }
            Some(agreement)
        }
        else if let Some(contract) = contract {
            if let Some(treaty) = treaty {
                report.error(MultipleAgreements.marked(treaty.pos));
                return Err(Failed)
            }
            Some(contract.into_agreement(AgreementType::Contract))
        }
        else if let Some(treaty) = treaty {
            Some(treaty.into_agreement(AgreementType::Treaty))
        }
        else {
            None
        };

        Ok(Basis {
            date: date?,
            document: document?,
            source: source?,
            agreement,
            note: note?,
        })
    }
}


//------------ Agreement -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Agreement {
    pub agreement_type: AgreementType,
    pub parties: List<Marked<entity::Link>>,
}

impl FromYaml<StoreLoader> for Agreement {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let agreement_type = value.take("type", context, report);
        let parties = value.take("parties", context, report);
        value.exhausted(report)?;

        Ok(Agreement {
            agreement_type: agreement_type?,
            parties: parties?
        })
    }
}



//------------ AgreementType -------------------------------------------------

data_enum! {
    pub enum AgreementType {
        { Contract: "contract" }
        { Treaty: "treaty" }
    }
}


//------------ Contract ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Contract {
    pub parties: List<Marked<entity::Link>>,
    pub pos: Location,
}

impl Contract {
    pub fn into_agreement(self, agreement_type: AgreementType) -> Agreement {
        Agreement {
            agreement_type,
            parties: self.parties
        }
    }
}

impl FromYaml<StoreLoader> for Contract {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let pos = value.location();
        let mut value = value.into_mapping(report)?;
        let parties = value.take("parties", context, report);
        value.exhausted(report)?;
        Ok(Contract { parties: parties?, pos })
    }
}


//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="only one of 'agreement', 'contract', or 'treaty' allowed")]
pub struct MultipleAgreements;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="one of 'agreement', 'contract', or 'treaty' required")]
pub struct MissingAgreement;


