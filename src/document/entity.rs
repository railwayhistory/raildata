
use std::cmp;
use std::collections::HashSet;
use std::str::FromStr;
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{
    DataStore, DocumentLink, FullStore, StoreLoader, XrefsBuilder, XrefsStore
};
use crate::types::{
    CountryCode, EventDate, Key, LanguageText, LanguageCode, LocalText, List,
    Marked, Set,
};
use super::{entity, line, source};
use super::common::{Basis, Common, Progress};


//------------ Link ----------------------------------------------------------

pub use super::combined::EntityLink as Link;


//------------ Document ------------------------------------------------------

pub use super::combined::EntityDocument as Document;

impl<'a> Document<'a> {
}


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Data {
    link: entity::Link,

    pub common: Common,
    pub subtype: Marked<Subtype>,
    pub events: EventList,
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

    pub fn link(&self) -> entity::Link {
        self.link
    }

    pub fn local_name(&self, lang: LanguageCode) -> &str {
        for event in self.events.iter().rev() {
            if let Some(name) = event.name(lang) {
                return name
            }
        }
        for event in self.events.iter().rev() {
            if let Some(name) = event.name(LanguageCode::ENG) {
                return name
            }
        }
        self.key()
    }

    pub fn local_short_name(&self, lang: LanguageCode) -> &str {
        for event in self.events.iter().rev() {
            if let Some(name) = event.short_name(lang) {
                return name
            }
        }
        for event in self.events.iter().rev() {
            if let Some(name) = event.short_name(LanguageCode::ENG) {
                return name
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
                if let Some(name) = event.prop(|record| {
                    record.properties.name.as_ref()
                }) {
                    if let Some(name) = name.for_language(lang) {
                        local = Some(name)
                    }
                    else {
                        default = Some(name.first())
                    }
                }
                continue
            }
            if let Some(name) = event.prop(|record| {
                record.properties.name.as_ref()
            }) {
                if let Some(name) = name.for_language(lang) {
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
                if let Some(name) = event.prop(|record| {
                    record.properties.short_name.as_ref()
                }) {
                    if let Some(name) = name.for_language(lang) {
                        local = Some(name)
                    }
                    else {
                        default = Some(name.first())
                    }
                }
                continue
            }
            if let Some(name) = event.prop(|record| {
                record.properties.short_name.as_ref()
            }) {
                if let Some(name) = name.for_language(lang) {
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

    fn event_records(&self) -> impl Iterator<Item = &EventRecord> + '_ {
        self.events.iter().flat_map(|ev| ev.records.iter())
    }

    /*
    fn event_records_rev(&self) -> impl Iterator<Item = &EventRecord> + '_ {
        self.events.iter().rev().map(|ev| ev.records.iter()).flatten()
    }
    */
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

    pub fn catalogue(
        &self,
        builder: &mut CatalogueBuilder,
        _store: &FullStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        // Insert names.
        let mut names = HashSet::new();
        for event in self.event_records() {
            if let Some(some) = event.properties.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
            if let Some(some) = event.properties.short_name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            builder.insert_name(name.into(), self.link.into())
        }

        // Insert countries
        if matches!(self.subtype.into_value(), Subtype::Country) {
            if let Ok(code) = CountryCode::from_str(
                &self.key().as_str()[4..]
            ) {
                builder.insert_country(code, self.link);
            }
        }

        Ok(())
    }
}


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Xrefs {
    pub line_regions: List<(line::Link, line::Section)>,

    /// All the sources that refer to this entity.
    pub source_regards: Set<source::Link>,

    /// All the sources that this entity has authored.
    pub source_author: Set<source::Link>,

    /// All the sources that this entity has edited.
    pub source_editor: Set<source::Link>,

    /// All the sources that this entity was responsible for as an organization.
    pub source_organization: Set<source::Link>,

    /// All the sources that this entity has published.
    pub source_publisher: Set<source::Link>,
}

impl Xrefs {
    pub fn source_regards_mut(&mut self) -> &mut Set<source::Link> {
        &mut self.source_regards
    }

    pub fn finalize(&mut self, store: &DataStore) {
        self.line_regions.sort_by(|left, right| {
            left.0.data(store).code().cmp(right.0.data(store).code())
        })
    }
}



//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Meta {
    pub current: Properties,
}

impl Meta {
    pub fn generate(
        data: &Data, _store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        let mut current = Properties::default();
        for record in data.event_records() {
            current.merge(&record.properties);
        }

        Ok(Meta {
            current
        })
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

#[derive(Clone, Debug)]
pub struct Event {
    pub date: EventDate,
    pub records: List<EventRecord>,
}

impl Event {
    pub fn name(&self, lang: LanguageCode) -> Option<&str> {
        LocalText::iter_for_language(
            self.records.iter().filter_map(|record| {
                record.properties.name.as_ref()
            }),
            lang
        )
    }

    pub fn short_name(&self, lang: LanguageCode) -> Option<&str> {
        LocalText::iter_for_language(
            self.records.iter().filter_map(|record| {
                record.properties.short_name.as_ref()
            }),
            lang
        )
    }

    fn prop<F: Fn(&EventRecord) -> Option<&T>, T: ?Sized>(
        &self, op: F
    ) -> Option<&T> {
        self.records.iter().find_map(op)
    }
}

impl FromYaml<StoreLoader> for Event {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;

        let date = value.take_default("date", context, report);
        let records = match value.take_opt("records", context, report) {
            Ok(Some(records)) => Ok(records),
            Ok(None) => {
                EventRecord::from_mapping(
                    &mut value, context, report
                ).map(List::with_value)
            }
            Err(err) => Err(err), 
        };

        value.exhausted(report)?;

        Ok(Event {
            date: date?,
            records: records?,
        })
    }
}


//------------ EventRecord ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct EventRecord {
    pub date: Option<EventDate>,
    pub document: List<Marked<source::Link>>,
    pub source: List<Marked<source::Link>>,
    pub basis: List<Basis>,
    pub note: Option<LanguageText>,

    pub property: Option<Property>,

    pub properties: Properties,
}

impl EventRecord {
    fn from_mapping(
        value: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let date = value.take_opt("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);
        let property = value.take_opt("property", context, report);
        let properties = Properties::from_mapping(value, context, report);

        Ok(Self {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,

            property: property?,

            properties: properties?,
        })
    }
}

impl FromYaml<StoreLoader> for EventRecord {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let res = Self::from_mapping(&mut value, context, report);
        value.exhausted(report)?;
        res
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Properties {
    /// The place of domicile of an organization.
    ///
    /// For geographic organizations, this is their capital.
    pub domicile: Option<List<Marked<entity::Link>>>,
    pub name: Option<LocalText>,
    pub owner: Option<List<Marked<entity::Link>>>,
    pub short_name: Option<LocalText>,
    pub status: Option<Marked<Status>>,
    pub successor: Option<Marked<entity::Link>>,

    /// An organization this organization is a unit of.
    pub superior: Option<List<Marked<entity::Link>>>,
}

impl  Properties {
    fn from_mapping(
        value: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let domicile = value.take_opt("domicile", context, report);
        let name = value.take_opt("name", context, report);
        let owner = value.take_default("owner", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let status = value.take_opt("status", context, report);
        let successor = value.take_opt("successor", context, report);
        let superior = value.take_opt("superior", context, report);
        Ok(Properties {
            domicile: domicile?,
            name: name?,
            owner: owner?,
            short_name: short_name?,
            status: status?,
            successor: successor?,
            superior: superior?,
        })
    }

    fn merge(&mut self, other: &Self) {
        if let Some(domicile) = other.domicile.as_ref() {
            self.domicile = Some(domicile.clone())
        }
        if let Some(name) = other.name.as_ref() {
            self.name = Some(name.clone())
        }
        if let Some(owner) = other.owner.as_ref() {
            self.owner = Some(owner.clone())
        }
        if let Some(name) = other.short_name.as_ref() {
            self.short_name = Some(name.clone())
        }
        if let Some(status) = other.status {
            self.status = Some(status)
        }
        if let Some(successor) = other.successor {
            self.successor = Some(successor)
        }
        if let Some(superior) = other.superior.as_ref() {
            self.superior = Some(superior.clone())
        }
    }
}


//------------ Property ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Property {
    pub role: Marked<PropertyRole>,
    pub region: List<Marked<entity::Link>>,
    pub constructor: List<Marked<entity::Link>>,
    pub operator: List<Marked<entity::Link>>,
    pub owner: List<Marked<entity::Link>>,
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

