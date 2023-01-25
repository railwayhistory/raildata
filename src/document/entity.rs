
use std::cmp;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{StoreLoader, XrefsBuilder, XrefsStore};
use crate::types::{
    EventDate, Key, LanguageText, LanguageCode, LocalText, List, Marked, Set
};
use super::common::{Basis, Common, Progress};
use super::{DocumentLink, LineLink, EntityLink, SourceLink};


//------------ Link ----------------------------------------------------------

pub use super::combined::EntityLink as Link;

//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub link: EntityLink,

    // Attributes
    pub common: Common,
    pub subtype: Marked<Subtype>,
    pub events: EventList,

    // Crosslinks
    pub line_region: List<LineLink>,
    pub source_author: Set<SourceLink>,
    pub source_editor: Set<SourceLink>,
    pub source_organization: Set<SourceLink>,
    pub source_publisher: Set<SourceLink>,
}

impl Data {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    pub fn name(&self, lang: LanguageCode) -> &str {
        self.local_short_name(lang)
    }

    pub fn local_name(&self, lang: LanguageCode) -> &str {
        for event in &self.events {
            if let Some(ref name) = event.name {
                if let Some(ref name) = name.for_language(lang) {
                    return name
                }
            }
        }
        for event in &self.events {
            if let Some(ref name) = event.name {
                if let Some(ref name) = name.for_language(LanguageCode::ENG) {
                    return name
                }
            }
        }
        self.key()
    }

    pub fn local_short_name(&self, lang: LanguageCode) -> &str {
        for event in &self.events {
            if let Some(ref name) = event.short_name {
                if let Some(ref name) = name.for_language(lang) {
                    return name
                }
            }
        }
        for event in &self.events {
            if let Some(ref name) = event.short_name {
                if let Some(ref name) = name.for_language(LanguageCode::ENG) {
                    return name
                }
            }
        }
        self.local_name(lang)
    }

    pub fn historic_name(
        &self, lang: LanguageCode, date: &EventDate
    ) -> &str {
        let mut local = None;
        let mut default = None;
        for event in self.events.iter().rev() {
            if date.sort_cmp(&event.date) == cmp::Ordering::Greater {
                if let Some(ref name) = event.name {
                    if let Some(name) = name.for_language(lang) {
                        local = Some(name)
                    }
                    else {
                        default = Some(name.first())
                    }
                }
                continue
            }
            if let Some(ref name) = event.name {
                if let Some(ref name) = name.for_language(lang) {
                    return name
                }
                else {
                    default = Some(name.first())
                }
            }
        }
        if let Some(local) = local {
            local
        }
        else if let Some(default) = default {
            default
        }
        else {
            self.key()
        }
    }

    pub fn historic_short_name(
        &self, lang: LanguageCode, date: &EventDate
    ) -> &str {
        let mut local = None;
        let mut default = None;
        for event in self.events.iter().rev() {
            if date.sort_cmp(&event.date) == cmp::Ordering::Greater {
                if let Some(ref name) = event.short_name {
                    if let Some(name) = name.for_language(lang) {
                        local = Some(name)
                    }
                    else {
                        default = Some(name.first())
                    }
                }
                continue
            }
            if let Some(ref name) = event.short_name {
                if let Some(ref name) = name.for_language(lang) {
                    return name
                }
                else {
                    default = Some(name.first())
                }
            }
        }
        if let Some(local) = local {
            local
        }
        else if let Some(default) = default {
            default
        }
        else {
            self.historic_name(lang, date)
        }
    }
}

impl Data {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        link: DocumentLink,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take("subtype", context, report);
        let events = doc.take("events", context, report);
        doc.exhausted(report)?;

        let mut res = Data {
            link: link.into(),

            common: common?,
            subtype: subtype?,
            events: events?,

            line_region: List::new(),
            source_author: Set::new(),
            source_editor: Set::new(),
            source_organization: Set::new(),
            source_publisher: Set::new(),
        };
        res.events.sort_by(|left, right| left.date.sort_cmp(&right.date));
        Ok(res)
    }

    pub fn xrefs(
        &self, 
        _builder: &mut XrefsBuilder,
        _store: &crate::store::DataStore,
        _report: &mut crate::load::report::PathReporter,
    ) -> Result<(), Failed> {
        Ok(())
    }

    pub fn process_names<F: FnMut(String)>(&self, mut process: F) {
        let mut names = HashSet::new();
        for event in &self.events {
            if let Some(some) = event.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
            if let Some(some) = event.short_name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            process(name.into())
        }
    }
}


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Xrefs;


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta;

impl Meta {
    pub fn generate(
        _data: &Data, _store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        Ok(Meta)
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
        { Placeholder: "placeholder" }
    }
}

impl Subtype {
    pub fn is_country(self) -> bool {
        matches!(self, Subtype::Country)
    }

    pub fn is_geographical(self) -> bool {
        matches!(self, Subtype::Country | Subtype::Place | Subtype::Region)
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

    //--- Organization property attributes
    //
    /// The place of domicile of an organization.
    ///
    /// For geographic organizations, this is their capital.
    pub domicile: List<Marked<EntityLink>>,
    pub name: Option<LocalText>,
    pub owner: Option<List<Marked<EntityLink>>>,
    pub property: Option<Property>,
    pub short_name: Option<LocalText>,
    pub status: Option<Marked<Status>>,
    pub successor: Option<Marked<EntityLink>>,

    /// An organization this organization is a unit of.
    pub superior: Option<Marked<EntityLink>>,
}

impl FromYaml<StoreLoader> for Event {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
       let date = value.take("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);
        let domicile = value.take_default("domicile", context, report);
        let name = value.take_opt("name", context, report);
        let owner = value.take_default("owner", context, report);
        let property = value.take_opt("property", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let status = value.take_opt("status", context, report);
        let successor = value.take_opt("successor", context, report);
        let superior = value.take_opt("superior", context, report);
        value.exhausted(report)?;
        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,
            domicile: domicile?,
            name: name?,
            owner: owner?,
            property: property?,
            short_name: short_name?,
            status: status?,
            successor: successor?,
            superior: superior?,
        })
    }
}


//------------ Property ------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub role: Marked<PropertyRole>,
    pub region: List<Marked<EntityLink>>,
    pub constructor: List<Marked<EntityLink>>,
    pub operator: List<Marked<EntityLink>>,
    pub owner: List<Marked<EntityLink>>,
}

impl FromYaml<StoreLoader> for Property {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let role = value.take("role", context, report);
        let region = value.take_default("region", context, report);
        let constructor = value.take_default("constructor", context, report);
        let owner = value.take_default("owner", context, report);
        let operator = value.take_default("operator", context, report);
        value.exhausted(report)?;
        Ok(Property {
            role: role?,
            region: region?,
            constructor: constructor?,
            owner: owner?,
            operator: operator?,
        })
    }
}


//------------ PropertyRole --------------------------------------------------

data_enum! {
    pub enum PropertyRole {
        { Constructor: "constructor" }
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


//------------ Crosslinks ----------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Crosslink {
    /// Lines related to this organization.
    ///
    /// The list is ordered by line code.
    pub lines: Vec<LineCrossref>,
}


//------------ LineCrossref --------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineCrossref {
    pub line: LineLink,
    pub region: bool,
    pub owned: bool,
    pub operated: bool,
}

