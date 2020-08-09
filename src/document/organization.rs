
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::catalogue::Catalogue;
use crate::library::{LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{EventDate, Key, LanguageText, LocalText, List, Marked, Set};
use super::common::{Basis, Common, Progress};
use super::{DocumentLink, OrganizationLink, SourceLink};


//------------ Organization --------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Organization {
    // Attributes
    pub common: Common,
    pub subtype: Marked<Subtype>,
    pub events: EventList,

    // Crosslinks
    pub source_author: Set<SourceLink>,
    pub source_editor: Set<SourceLink>,
    pub source_organization: Set<SourceLink>,
    pub source_publisher: Set<SourceLink>,
}

impl Organization {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }
}

impl Organization {
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
        Ok(Organization {
            common: common?,
            subtype: subtype?,
            events: events?,
            source_author: Set::new(),
            source_editor: Set::new(),
            source_organization: Set::new(),
            source_publisher: Set::new(),
        })
    }

    pub fn crosslink(
        &self,
        _link: OrganizationLink,
        _library: &LibraryMut,
        _report: &mut StageReporter
    ) {
    }

    /*
    pub fn verify(&self, _report: &mut StageReporter) {
    }
    */

    pub fn catalogue(
        &self,
        link: OrganizationLink,
        catalogue: &mut Catalogue,
        _report: &mut StageReporter
    ) {
        let link = DocumentLink::from(link);
        // Names
        catalogue.insert_name(self.common.key.to_string(), link);
        let mut names = HashSet::new();
        for event in &self.events {
            if let Some(some) = event.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            catalogue.insert_name(name.into(), link)
        }

        // Countries
        if *self.subtype.as_value() == Subtype::Country {
            catalogue.push_country(link)
        }
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Company: "company" }
        { Country: "country" }
        { Person: "person" }
        { Place: "place" }
        { Region: "region" }
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
    pub basis: List<Basis>,
    pub note: Option<LanguageText>,

    // Organization property attributes
    pub domicile: List<Marked<OrganizationLink>>,
    pub master: Option<Marked<OrganizationLink>>,
    pub name: Option<LocalText>,
    pub owner: Option<List<Marked<OrganizationLink>>>,
    pub property: Option<Property>,
    pub short_name: Option<LocalText>,
    pub status: Option<Status>,
    pub successor: Option<Marked<OrganizationLink>>,
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
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);
        let domicile = value.take_default("domicile", context, report);
        let master = value.take_opt("master", context, report);
        let name = value.take_opt("name", context, report);
        let owner = value.take_default("owner", context, report);
        let property = value.take_opt("property", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let status = value.take_opt("status", context, report);
        let successor = value.take_opt("successor", context, report);
        value.exhausted(report)?;
        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,
            domicile: domicile?,
            master: master?,
            name: name?,
            owner: owner?,
            property: property?,
            short_name: short_name?,
            status: status?,
            successor: successor?,
        })
    }
}


//------------ Property ------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub role: Marked<PropertyRole>,
    pub constructor: List<Marked<OrganizationLink>>,
    pub operator: List<Marked<OrganizationLink>>,
    pub owner: List<Marked<OrganizationLink>>,
}

impl FromYaml<LibraryBuilder> for Property {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let role = value.take("role", context, report);
        let constructor = value.take_default("constructor", context, report);
        let owner = value.take_default("owner", context, report);
        let operator = value.take_default("operator", context, report);
        value.exhausted(report)?;
        Ok(Property {
            role: role?,
            constructor: constructor?,
            owner: owner?,
            operator: operator?,
        })
    }
}


//------------ PropertyRole --------------------------------------------------

data_enum! {
    pub enum PropertyRole {
        { Constructur: "constructor" }
        { Owner: "owner" }
        { Operator: "operator" }
    }
}


//------------ Status --------------------------------------------------------

data_enum! {
    pub enum Status {
        { Forming: "forming" }
        { Open: "open" }
        { Closed: "closed" }
    }
}

