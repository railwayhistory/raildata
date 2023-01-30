
use std::collections::{HashSet, HashMap};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::catalogue::CatalogueBuilder;
use crate::document::path::Coord;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{
    DataStore, DocumentLink, FullStore, StoreLoader, XrefsBuilder, XrefsStore,
};
use crate::types::{
    CountryCode, EventDate, IntoMarked, Key, LanguageCode, LanguageText, List,
    LocalText, Marked, Set,
};
use super::{line, path, point, source};
use super::common::{Basis, Common, Progress};


//------------ Link ----------------------------------------------------------

pub use super::combined::PointLink as Link;


//------------ Document ------------------------------------------------------

pub use super::combined::PointDocument as Document;


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    link: point::Link,

    pub common: Common,
    pub events: EventList,
    pub records: RecordList,
    pub junction: Option<Marked<bool>>,
    pub subtype: Marked<Subtype>,
}

/// # Data Access
///
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

    pub fn link(&self) -> point::Link {
        self.link
    }

    /// Returns whether the point can’t be a junction.
    ///
    /// This happens if it has the `junction` attribute set to false.
    pub fn is_never_junction(&self) -> bool {
        !self.junction.map(Marked::into_value).unwrap_or(true)
    }

    /// Returns the preferred name for the given language.
    pub fn name(&self, lang: LanguageCode) -> &str {
        for properties in
            self.events.iter().map(|item| &item.properties).rev().chain(
                self.records.iter().map(|item| &item.properties).rev()
            )
        {
            if let Some(ref name) = properties.name {
                if let Some(name) = name.for_language(lang) {
                    return name
                }
            }
            if let Some(ref name) = properties.designation {
                if let Some(name) = name.for_language(lang) {
                    return name
                }
            }
        }
        self.key().as_str()
    }

    /// Returns the current name.
    pub fn name_in_jurisdiction(
        &self, jurisdiction: Option<CountryCode>
    ) -> &str {
        if let Some(res) = self.events_then_records(|properties| {
            if let Some(ref name) = properties.name {
                name.for_jurisdiction(jurisdiction)
            }
            else if let Some(ref name) = properties.designation {
                name.for_jurisdiction(jurisdiction)
            }
            else {
                None
            }
        }) {
            return res.0
        }
        if let Some(res) = self.events_then_records(|properties| {
            if let Some(ref name) = properties.name {
                Some(name.first())
            }
            else if let Some(ref name) = properties.designation {
                Some(name.first())
            }
            else {
                None
            }
        }) {
            return res.0
        }
        self.key().as_str()
    }

    /// Returns the current location for the given line.
    ///
    /// If the point has a location on this line, returns the location as well
    /// as whether it has changed.
    pub fn location(&self, line: line::Link) -> Option<(Option<&str>, bool)> {
        self.events_then_records(|properties| {
            if properties.location.is_empty() {
                return None
            }
            for (link, loc) in properties.location.iter() {
                if link == line {
                    return Some(loc)
                }
            }
            None
        })
    }

    /// Returns the current category of the point and whether it has changed.
    pub fn category(
        &self
    ) -> Option<(impl Iterator<Item = Category> + '_, bool)> {
        self.events_then_records(
            |properties| properties.category.as_ref()
        ).map(|(res, changed)| {
            (res.iter().map(|cat| cat.into_value()), changed)
        })
    }

    /// Returns the current status.
    pub fn status(&self) -> Status {
        self.events_then_records(|properties| {
            properties.status.as_ref()
        }).map(|res| res.0.into_value()).unwrap_or(Status::Open)
    }

    /// Returns whether the point is currently open.
    pub fn is_open(&self) -> bool {
        self.status() == Status::Open
    }

    fn events_then_records<'a, F, R>(&'a self, mut op: F) -> Option<(R, bool)>
    where F: FnMut(&'a Properties) -> Option<R> {
        let mut res = None;
        let mut changed = false;
        for event in &self.events {
            if let Some(value) = op(&event.properties) {
                if res.is_some() {
                    changed = true
                }
                res = Some(value)
            }
        }
        if let Some(res) = res.take() {
            return Some((res, changed))
        }
        for record in &self.records {
            if let Some(value) = op(&record.properties) {
                if res.is_some() {
                    changed = true
                }
                res = Some(value)
            }
        }
        res.map(|res| (res, changed))
    }
}


/// # Loading
///
impl Data {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        link: DocumentLink,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let events = doc.take_opt("events", context, report);
        let records = doc.take_opt("records", context, report);
        let junction = doc.take_opt("junction", context, report);
        let subtype = doc.take_default("subtype", context, report);
        doc.exhausted(report)?;

        let mut events: EventList = events?.unwrap_or_default();
        events.sort_by(|left, right| left.date.sort_cmp(&right.date));
        let records: RecordList = records?.unwrap_or_default();

        Ok(Data {
            link: link.into(),
            common: common?,
            events,
            records,
            junction: junction?,
            subtype: subtype?,
        })
    }

    /*
    pub fn verify(&self, _report: &mut StageReporter) {
    }
    */

    pub fn xrefs(
        &self, 
        _builder: &mut XrefsBuilder,
        _store: &crate::store::DataStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        Ok(())
    }

    pub fn catalogue(
        &self,
        builder: &mut CatalogueBuilder,
        _store: &FullStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        let mut names = HashSet::new();
        self.events_then_records(|properties| {
            if let Some(some) = properties.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
            Some(())
        });
        for name in names {
            builder.insert_name(name.into(), self.link.into())
        }
        Ok(())
    }
}


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Xrefs {
    pub lines: List<line::Link>,
    pub source_regards: Set<source::Link>,
}

impl Xrefs {
    pub fn source_regards_mut(&mut self) -> &mut Set<source::Link> {
        &mut self.source_regards
    }

    pub fn finalize(&mut self, _store: &DataStore) {
    }
}


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub junction: bool,
    pub coord: Option<Coord>,
}

impl Meta {
    pub fn generate(
        data: &Data, store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        let xrefs = data.link.xrefs(store);

        // junction: Either explicitely set or if we are part of more than
        // one line or if we are the first or last point on the line.
        let junction = if let Some(value) = data.junction {
            value.into_value()
        }
        else if xrefs.lines.len() > 1 {
            true
        }
        else {
            match xrefs.lines.first() {
                Some(line) => {
                    let line = line.data(store);
                    line.points.first().map(|x| x.into_value())
                        == Some(data.link)
                    || line.points.last().map(|x| x.into_value())
                        == Some(data.link)
                }
                None => false
            }
        };

        // coord: Find the newest event that has a site attribute and take the
        // first entry.
        let mut coord = None;
        for event in data.events.iter().rev() {
            if let Some(site) = event.site.as_ref() {
                for item in site.0.iter() {
                    coord = item.0.data(store).get_coord(item.1.as_value());
                    if coord.is_some() {
                        break
                    }
                }
            }
        }

        Ok(Meta {
            junction,
            coord,
        })
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Border: "border" }
        { Break: "break" }
        { Post: "post" }
        { Reference: "reference" }

        default Post
    }
}

impl Subtype {
    pub fn is_post(self) -> bool {
        matches!(self, Subtype::Post)
    }
}


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub date: EventDate,
    pub document: List<Marked<source::Link>>,
    pub source: List<Marked<source::Link>>,
    pub basis: List<Basis>,
    pub note: Option<LanguageText>,

    pub split_from: Option<Marked<point::Link>>,
    pub merged: Option<Marked<point::Link>>,

    pub connection: Option<List<Marked<point::Link>>>,
    pub site: Option<Site>,

    pub properties: Properties,
}

impl FromYaml<StoreLoader> for Event {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;

        let date = value.take_default("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);

        let split_from = value.take_opt("split_from", context, report);
        let merged = value.take_opt("merged", context, report);

        let connection = value.take_opt("connection", context, report);
        let site = value.take_opt("site", context, report);

        let properties = Properties::from_yaml(&mut value, context, report);

        value.exhausted(report)?;

        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,

            merged: merged?,
            split_from: split_from?,

            connection: connection?,
            site: site?,

            properties: properties?,
        })
    }
}


//------------ RecordList ----------------------------------------------------

pub type RecordList = List<Record>;


//------------ Record --------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    pub document: List<Marked<source::Link>>,
    pub note: Option<LanguageText>,

    pub properties: Properties,
}

impl FromYaml<StoreLoader> for Record {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;

        let date = value.take_default("date", context, report);
        let document = value.take("document", context, report);
        let note = value.take_opt("note", context, report);

        let properties = Properties::from_yaml(&mut value, context, report);
        value.exhausted(report)?;

        let _: EventDate = date?;
        Ok(Record {
            document: document?,
            note: note?,
            properties: properties?,
        })
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Properties {
    pub status: Option<Marked<Status>>,

    pub name: Option<LocalText>,
    pub short_name: Option<LocalText>,
    pub public_name: Option<List<LocalText>>,
    pub designation: Option<LocalText>,
    pub de_name16: Option<DeName16>,

    pub category: Option<Set<Marked<Category>>>,
    pub de_rang: Option<Marked<DeRang>>,
    pub superior: Option<Option<List<Marked<point::Link>>>>,
    pub codes: Codes,

    pub location: Location,

    pub staff: Option<Staff>,
    pub service: Option<Marked<Service>>,
    pub passenger: Option<Marked<ServiceRate>>,
    pub luggage: Option<Marked<ServiceRate>>,
    pub express: Option<Marked<ServiceRate>>,
    pub goods: Option<Marked<ServiceRate>>,
}

impl Properties {
    fn from_yaml(
        value: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let pos = value.location();

        let status = value.take_opt("status", context, report);

        let name = value.take_opt("name", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let public_name = value.take_opt("public_name", context, report);
        let designation = value.take_opt("designation", context, report);
        let de_name16 = value.take_opt("de.name16", context, report);

        let category = value.take_opt("category", context, report);
        let de_rang = value.take_opt("de.rang", context, report);
        let superior = value.take_opt("superior", context, report);
        let codes = Codes::from_yaml(value, context, report);

        let location = value.take_default("location", context, report);

        let staff = value.take_opt("staff", context, report);
        let service = value.take_opt("service", context, report);
        let passenger = value.take_opt("passenger", context, report);
        let luggage = value.take_opt("luggage", context, report);
        let express = value.take_opt("express", context, report);
        let goods = value.take_opt("goods", context, report);

        let master = value.take_opt("master", context, report);

        let mut superior = superior?;
        if let Some(master) = master? {
            if superior.is_some() {
                report.error(SuperiorAndMaster.marked(pos));
                return Err(Failed);
            }
            else {
                superior = master;
            }
        }

        Ok(Properties {
            status: status?,
            name: name?,
            short_name: short_name?,
            public_name: public_name?,
            designation: designation?,
            de_name16: de_name16?,
            category: category?,
            de_rang: de_rang?,
            superior: superior,
            codes: codes?,
            location: location?,
            staff: staff?,
            service: service?,
            passenger: passenger?,
            luggage: luggage?,
            express: express?,
            goods: goods?,
        })
    }
}


//------------ Category ------------------------------------------------------

data_enum! {
    pub enum Category {
        { Border: "border" }
        { DeAbzw: "de.Abzw" }
        { DeAnst: "de.Anst" }
        { DeAwanst: "de.Awanst" }
        { DeBf: "de.Bf" }
        { DeBft: "de.Bft" }
        { DeBk: "de.Bk" }
        { DeDkst: "de.Dkst" }
        { DeGlgr: "de.Glgr" }
        { DeHp: "de.Hp" }
        { DeHst: "de.Hst" }
        { DeKr: "de.Kr" }
        { DeKrbf: "de.Krbf" }
        { DeKrst: "de.Krst" }
        { DeLdst: "de.Ldst" }
        { DeMuseum: "de.Museum" }
        { DePo: "de.Po" }
        { DeStrw: "de.Strw" }
        { DeStw: "de.Stw" }
        { DeUehst: "de.Ühst" }
        { DeUest: "de.Üst" }
        { DeAhst: "de.Ahst" }
        { DeGnst: "de.Gnst" }
        { DeGa: "de.Ga" }
        { DeUst: "de.Ust" }
        { DeTp: "de.Tp" }
        { DeEGr: "de.EGr" }
        { DeGp: "de.Gp" }
        { DeLGr: "de.LGr" }
        { DeRBGr: "de.RBGr" }

        { DkB: "dk.B" }         // Billetssalgssted
        { DkGr: "dk.Gr" }       // Grænse
        { DkSmd: "dk.Smd" }     // Sidespor med dækningssignal
        { DkSud: "dk.Sud" }     // Sidespor uden dækningssignal
        { DkSt: "dk.St" }       // Station
        { DkT: "dk.T" }         // Trinbræt
        { DkTs: "dk.Ts" }       // Teknisk station
        { DkVm: "dk.VM" }       // VM-station

        { NlAansl: "nl.Aansl" }
        { NlGem: "nl.Gem" }
        { NlH: "nl.H" }
        { NlKnp: "nl.Knp" }
        { NlOlp: "nl.Olp" }
        { NlSt: "nl.St" }

        { NoS: "no.s" }
        { NoSp: "no.sp" }
        { NoHp: "no.hp" }
    }
}

impl Category {
    pub fn code(self) -> &'static str {
        use self::Category::*;

        match self {
            Border => "border",
            DeAbzw => "Abzw",
            DeAnst => "Anst",
            DeAwanst => "Awanst",
            DeBf => "Bf",
            DeBft => "Bft",
            DeBk => "Bk",
            DeDkst => "Dkst",
            DeGlgr => "Glgr",
            DeHp => "Hp",
            DeHst => "Hst",
            DeKr => "Kr",
            DeKrbf => "Krbf",
            DeKrst => "Krst",
            DeLdst => "Ldst",
            DeMuseum => "Museum",
            DePo => "Po",
            DeStrw => "Strw",
            DeStw => "Stw",
            DeUehst => "Ühst",
            DeUest => "Üst",
            DeAhst => "Ahst",
            DeGnst => "Gnst",
            DeGa => "Ga",
            DeUst => "Ust",
            DeTp => "Tp",
            DeEGr => "EGr",
            DeGp => "Gp",
            DeLGr => "LGr",
            DeRBGr => "RBGr",

            DkB => "B",
            DkGr => "Gr",
            DkSmd => "Smd",
            DkSud => "Sud",
            DkSt => "St",
            DkT => "T",
            DkTs => "Ts",
            DkVm => "VM",

            NlAansl => "Aansluiting",
            NlGem => "Goederen Emplacement",
            NlH => "Halte",
            NlKnp => "Knooppunt",
            NlOlp => "Overloop",
            NlSt => "Station",

            NoS => "S",
            NoSp => "Sp",
            NoHp => "Hp",
        }
    }
}


//------------ Location ------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Location(List<(Marked<line::Link>, Option<Marked<String>>)>);

impl Location {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> Option<(line::Link, Option<&str>)> {
        self.0.first().map(|(link, value)| {
            (link.into_value(), value.as_ref().map(|value| value.as_str()))
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (line::Link, Option<&str>)> {
        self.0.iter().map(|item| {
            (item.0.into_value(), item.1.as_ref().map(|s| s.as_str()))
        })
    }
}

impl FromYaml<StoreLoader> for Location {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut res = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(report)?.into_iter() {
            let key = match Marked::from_string(key, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue;
                }
            };
            let key = line::Link:: build(key, context, report);
            if value.is_null() {
                res.push((key, None))
            }
            else if let Ok(value) = value.into_string(report) {
                res.push((key, Some(value)))
            }
            else {
                err = true
            }
        }
        if err {
            Err(Failed)
        }
        else {
            Ok(Location(res))
        }
    }
}


//------------ Service -------------------------------------------------------

data_enum! {
    pub enum Service {
        { Full: "full" }
        { None: "none" }
        { Passenger: "passenger" }
        { Goods: "goods" }
    }
}


//------------ ServiceRate ---------------------------------------------------

data_enum! {
    pub enum ServiceRate {
        { None: "none" }
        { Limited: "limited" }
        { Full: "full" }
    }
}


//------------ ServiceSet ----------------------------------------------------

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServiceSet {
    pub passenger: Option<ServiceRate>,
    pub luggage: Option<ServiceRate>,
    pub express: Option<ServiceRate>,
    pub goods: Option<ServiceRate>,
}

impl ServiceSet {
    pub fn is_some(&self) -> bool {
        self.passenger.is_some() || self.luggage.is_some()
            || self.express.is_some() || self.goods.is_some()
    }
}

impl From<Service> for ServiceSet {
    fn from(service: Service) -> ServiceSet {
        match service {
            Service::Full => {
                ServiceSet {
                    passenger: Some(ServiceRate::Full),
                    luggage: Some(ServiceRate::Full),
                    express: Some(ServiceRate::Full),
                    goods: Some(ServiceRate::Full),
                }
            }
            Service::None => {
                ServiceSet {
                    passenger: Some(ServiceRate::None),
                    luggage: Some(ServiceRate::None),
                    express: Some(ServiceRate::None),
                    goods: Some(ServiceRate::None),
                }
            }
            Service::Passenger => {
                ServiceSet {
                    passenger: Some(ServiceRate::Full),
                    luggage: Some(ServiceRate::None),
                    express: Some(ServiceRate::None),
                    goods: Some(ServiceRate::None),
                }
            }
            Service::Goods => {
                ServiceSet {
                    passenger: Some(ServiceRate::None),
                    luggage: Some(ServiceRate::None),
                    express: Some(ServiceRate::None),
                    goods: Some(ServiceRate::Full),
                }
            }
        }
    }
}

impl<'a> From<&'a Properties> for ServiceSet {
    fn from(properties: &'a Properties) -> ServiceSet {
        let mut res = properties.service.map(|s|
            s.into_value().into()
        ).unwrap_or_else(ServiceSet::default);
        
        if let Some(rate) = properties.passenger {
            res.passenger = Some(rate.into_value())
        }
        if let Some(rate) = properties.luggage {
            res.luggage = Some(rate.into_value())
        }
        if let Some(rate) = properties.express {
            res.express = Some(rate.into_value())
        }
        if let Some(rate) = properties.goods {
            res.goods = Some(rate.into_value())
        }

        res
    }
}


//------------ Side ----------------------------------------------------------

data_enum! {
    pub enum Side {
        { Left: "left" }
        { Right: "right" }
        { Up: "up" }
        { Down: "down" }
        { Center: "center" }
    }
}


//------------ Site ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Site(pub List<(Marked<path::Link>, Marked<String>)>);

impl FromYaml<StoreLoader> for Site {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut res = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(report)?.into_iter() {
            let key = match Marked::from_string(key, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue;
                }
            };
            let key = path::Link::build(key, context, report);
            match value.into_string(report) {
                Ok(value) => res.push((key, value)),
                Err(_) => { err = true }
            }
        }
        if err {
            Err(Failed)
        }
        else {
            Ok(Site(res))
        }
    }
}


//------------ Staff ---------------------------------------------------------

data_enum! {
    pub enum Staff {
        { Full: "full" }
        { Agent: "agent" }
        { None: "none" }
    }
}


//------------ Status -------------------------------------------------------

data_enum! {
    pub enum Status {
        { Planned: "planned" }
        { Construction: "construction" }
        { Open: "open" }
        { Suspended: "suspended" }
        { Reopened: "reopened" }
        { Closed: "closed" }
    }
}


//------------ DeRang --------------------------------------------------------

data_enum! {
    pub enum DeRang {
        { I: "I" }
        { Ii: "II" }
        { Iii: "III" }
        { Iiia: "IIIa" }
        { Iiib: "IIIb" }
        { Iv: "IV" }
        { V: "V" }
        { Vi: "VI" }
        { U: "U" }
        { S: "S" }
    }
}


//------------ DeName16 ------------------------------------------------------

pub type DeName16 = Marked<String>;


//------------ Codes ---------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Codes {
    codes: HashMap<CodeType, List<Marked<String>>>,
}

impl Codes {
    pub fn iter(
        &self
    ) -> impl Iterator<Item = (CodeType, impl Iterator<Item = &str>)> + '_ {
        self.codes.iter().map(|(key, value)| {
            (*key, value.iter().map(|item| item.as_str()))
        })
    }
}

impl Codes {
    fn from_yaml(
        value: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut err = false;
        let mut res = HashMap::new();

        for &item in CodeType::ALL {
            let code = match value.take_opt(item.as_str(), context, report) {
                Ok(Some(Some(code))) => code,
                Ok(Some(None)) => List::default(),
                Ok(None) => continue,
                Err(_) => {
                    err = true;
                    continue
                }
            };

            for value in &code {
                if item.check_value(value, report).is_err() {
                    err = true
                }
            }

            if !err {
                res.insert(item, code);
            }
        }

        if err {
            Err(Failed)
        }
        else {
            Ok(Codes { codes: res })
        }
    }
}


//------------ CodeType ------------------------------------------------------

data_enum! {
    pub enum CodeType {
        { Plc: "PLC" }
        { DeDs100: "de.DS100" }
        { DeDstnr: "de.dstnr" }
        { DeLknr: "de.lknr" }
        { DeVbl: "de.VBL" }
        { DkRef: "dk.ref" }
        { NlAfk: "nl.afk" }
        { NoFs: "no.fs" }
        { NoNjk: "no.NJK" }
        { NoNsb: "no.NSB" }
    }
}

impl CodeType {
    fn check_value(
        self, _value: &Marked<String>, _report: &mut PathReporter
    ) -> Result<(), Failed> {
        Ok(())
    }
}


//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="only one of 'superior' and 'master' allowed")]
pub struct SuperiorAndMaster;

