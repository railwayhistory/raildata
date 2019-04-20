
use std::ops;
use std::str::FromStr;
use crate::library::{LibraryBuilder, LibraryMut, Library};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{
    Date, EventDate, IntoMarked, Key, LanguageText, List, LocalText, Location,
    Marked, Set
};
use super::{LineLink, OrganizationLink, PathLink, Point, PointLink, SourceLink};
use super::common::{Alternative, Basis, Common, Contract, Progress};

//mod verify;

//------------ Line ----------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Line {
    pub common: Common,
    pub label: Set<Label>,
    pub note: Option<LanguageText>,
    pub events: EventList,
    pub points: Points,
}

impl Line {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    pub fn code(&self) -> Result<(&str, &str), &str> {
        let code = self.key().as_str();
        if code.starts_with("line.") && code.get(7..8) == Some(".") {
            Ok((&code[5..7], &code[8..]))
        }
        else {
            Err(code)
        }
    }

    fn last_junction_index(&self, library: &Library) -> usize {
        self.points.iter().enumerate().rev().map(|(idx, point)| {
            (idx, point.follow(library))
        }).find_map(|(idx, point)| {
            if !point.is_never_junction() {
                Some(idx)
            }
            else {
                None
            }
        }).unwrap_or_else(|| self.points.len() - 1)
    }

    pub fn junctions<'a>(
        &'a self, library: &'a Library
    ) -> impl Iterator<Item=&'a Point> + 'a {
        let mut first = true;
        let last = self.last_junction_index(library);
        self.points.iter().enumerate().filter_map(move |(idx, point)| {
            let point = point.follow(library);
            if first {
                if !point.is_never_junction() {
                    first = false;
                    Some(point)
                }
                else {
                    None
                }
            }
            else if idx == last {
                Some(point)
            }
            else if idx > last {
                None
            }
            else if point.is_junction() {
                Some(point)
            }
            else {
                None
            }
        })
    }
}

impl Line {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let label = doc.take_default("label", context, report);
        let note = doc.take_opt("note", context, report);
        let events = doc.take("events", context, report);
        let points = doc.take("points", context, report);
        doc.exhausted(report)?;
        Ok(Line {
            common: common?,
            label: label?,
            note: note?,
            events: events?,
            points: points?,
        })
    }

    pub fn crosslink(
        &self,
        link: LineLink,
        library: &LibraryMut,
        _report: &mut StageReporter
    ) {
        for point in self.points.iter() {
            point.update(library, move |point| point.add_line(link))
        }
    }

/*
    pub fn verify(&self, report: &mut StageReporter) {
        verify::verify(self, report)
    }
*/
}


//------------ Label ---------------------------------------------------------

data_enum! {
    pub enum Label {
        { Connection: "connection" }
        { Freight: "freight" }
        { Port: "port" }
        { DeSBahn: "de.S-Bahn" }
    }
}


//------------ Points --------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Points {
    pub points: Vec<Marked<PointLink>>,
    pub indexes: Vec<(PointLink, usize)>,
}

impl Points {
    pub fn get_index(&self, link: &PointLink) -> Option<usize> {
        self.indexes.binary_search_by(|x| link.cmp(&x.0)).ok()
    }
}

impl ops::Deref for Points {
    type Target = [Marked<PointLink>];

    fn deref(&self) -> &Self::Target {
        self.points.as_ref()
    }
}

impl FromYaml<LibraryBuilder> for Points {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let points: Vec<Marked<PointLink>> = Vec::from_yaml(
            value, context, report
        )?;
        let mut indexes: Vec<_> = points.iter().enumerate()
            .map(|(index, point)| (point.as_value().clone(), index))
            .collect();
        indexes.sort_unstable_by(|x, y| x.0.cmp(&y.0));
        Ok(Points { points, indexes })
    }
}


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Event {
    pub date: EventDate,
    pub sections: List<Section>,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
    pub alternative: List<Alternative>,
    pub basis: List<Basis>,
    pub note: Option<LanguageText>,

    pub concession: Option<Concession>,
    pub expropriation: Option<Concession>,
    pub contract: Option<Contract>,
    pub treaty: Option<Contract>,

    pub category: Option<Set<Category>>,
    pub constructor: Option<List<Marked<OrganizationLink>>>,
    pub course: Option<List<CourseSegment>>,
    pub electrified: Option<Option<Set<Electrified>>>,
    pub freight: Option<Freight>,
    pub gauge: Option<Set<Gauge>>,
    pub local_name: Option<LocalText>, // XXX Drop
    pub name: Option<LocalText>,
    pub operator: Option<List<Marked<OrganizationLink>>>,
    pub owner: Option<List<Marked<OrganizationLink>>>,
    pub passenger: Option<Passenger>,
    pub rails: Option<Marked<u8>>,
    pub region: Option<List<Marked<OrganizationLink>>>,
    pub reused: Option<List<Marked<LineLink>>>,
    pub status: Option<Status>,
    pub tracks: Option<Marked<u8>>,

    pub de_vzg: Option<DeVzg>,
}

impl Event {
}

impl FromYaml<LibraryBuilder> for Event {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take("date", context, report);
        let sections = value.take_default("sections", context, report);
        let start = value.take_opt("start", context, report);
        let end = value.take_opt("end", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let alternative = value.take_default("alternative", context, report);
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);

        let concession = value.take_opt("concession", context, report);
        let expropriation = value.take_opt("expropriation", context, report);
        let contract = value.take_opt("contract", context, report);
        let treaty = value.take_opt("treaty", context, report);
        
        let category = value.take_opt("category", context, report);
        let constructor = value.take_opt("constructor", context, report);
        let course = value.take_default("course", context, report);
        let electrified = value.take_opt("electrified", context, report);
        let freight = value.take_opt("freight", context, report);
        let gauge = value.take_opt("gauge", context, report);
        let local_name = value.take_opt("local_name", context, report);
        let name = value.take_opt("name", context, report);
        let operator = value.take_opt("operator", context, report);
        let owner = value.take_opt("owner", context, report);
        let passenger = value.take_opt("passenger", context, report);
        let rails = value.take_opt("rails", context, report);
        let region = value.take_opt("region", context, report);
        let reused = value.take_opt("reused", context, report);
        let status = value.take_opt("status", context, report);
        let tracks = value.take_opt("tracks", context, report);

        let de_vzg = value.take_opt("de.VzG", context, report);

        value.exhausted(report)?;

        let mut sections: List<Section> = sections?;
        let start: Option<Marked<PointLink>> = start?;
        let end: Option<Marked<PointLink>> = end?;
        match (start, end) {
            (None, None) => { },
            (start, end) => {
                if !sections.is_empty() {
                    if let Some(start) = start {
                        report.error(
                            StartWithSections.marked(start.location())
                        );
                    }
                    if let Some(end) = end {
                        report.error(EndWithSections.marked(end.location()));
                    }
                    return Err(Failed)
                }
                sections.push(Section { start, end })
            }
        };
        
        Ok(Event {
            date: date?,
            sections: sections,
            document: document?,
            source: source?,
            alternative: alternative?,
            basis: basis?,
            note: note?,

            concession: concession?,
            expropriation: expropriation?,
            contract: contract?,
            treaty: treaty?,

            category: category?,
            constructor: constructor?,
            course: course?,
            electrified: electrified?,
            freight: freight?,
            gauge: gauge?,
            local_name: local_name?,
            name: name?,
            operator: operator?,
            owner: owner?,
            passenger: passenger?,
            rails: rails?,
            region: region?,
            reused: reused?,
            status: status?,
            tracks: tracks?,

            de_vzg: de_vzg?,
        })
    }
}


//------------ Section -------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Section {
    pub start: Option<Marked<PointLink>>,
    pub end: Option<Marked<PointLink>>,
}

impl FromYaml<LibraryBuilder> for Section {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let start = value.take_opt("start", context, report);
        let end = value.take_opt("end", context, report);
        value.exhausted(report)?;
        Ok(Section {
            start: start?,
            end: end?,
        })
    }
}


//------------ Category ------------------------------------------------------

data_enum! {
    pub enum Category {
        { DeHauptbahn: "de.Hauptbahn" }
        { DeNebenbahn: "de.Nebenbahn" }
        { DeKleinbahn: "de.Kleinbahn" }
        { DeAnschl: "de.Anschl" }
        { DeBfgleis: "de.Bfgleis" }
        { DeStrab: "de.Strab" }
    }
}


//------------ Concession ----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Concession {
    pub by: List<Marked<OrganizationLink>>,
    pub to: List<Marked<OrganizationLink>>,
    pub until: Option<Marked<Date>>,
}


impl FromYaml<LibraryBuilder> for Concession {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let by = value.take_default("by", context, report);
        let to = value.take_default("for", context, report);
        let until = value.take_opt("until", context, report);
        value.exhausted(report)?;
        Ok(Concession { by: by?, to: to?, until: until? })
    }
}


//------------ CourseSegment -------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct CourseSegment {
    pub path: Marked<PathLink>,
    pub start: Marked<String>,
    pub end: Marked<String>,
}

impl FromYaml<LibraryBuilder> for CourseSegment {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let (value, location) = value.into_string(report)?.unwrap();
        let mut value = value.split_whitespace();
        let path = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let key = match Key::from_str(path) {
            Ok(key) => key.marked(location),
            Err(err) => {
                report.error(err.marked(location));
                return Err(Failed)
            }
        };
        let path = PathLink::build(key, context, report);
        let start = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let start = Marked::new(String::from(start), location);
        let end = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let end = Marked::new(String::from(end), location);
        if value.next().is_some() {
            report.error(InvalidCourseSegment.marked(location));
            return Err(Failed)
        }
        Ok(CourseSegment { path, start, end })
    }
}


//------------ Electrified ---------------------------------------------------

pub type Electrified = Marked<String>;


//------------ Freight -------------------------------------------------------

data_enum! {
    pub enum Freight {
        { None: "none" }
        { Restricted: "restricted" }
        { Full: "full" }
    }
}


//------------ Gauge ---------------------------------------------------------

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct Gauge(pub Marked<u16>);

impl Gauge {
    pub fn gauge(&self) -> u16 {
        self.0.to_value()
    }

    pub fn location(&self) -> Location {
        self.0.location()
    }
}

impl Default for Gauge {
    fn default() -> Gauge {
        Gauge(Marked::from_value(1435))
    }
}

impl<C> FromYaml<C> for Gauge {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let (value, location) = value.into_string(report)?.unwrap();
        if !value.ends_with("mm") {
            report.error(InvalidGauge.marked(location));
            return Err(Failed)
        }
        match u16::from_str(&value[0..value.len() - 2]) {
            Ok(value) => Ok(Gauge(Marked::new(value, location))),
            Err(_) => {
                report.error(InvalidGauge.marked(location));
                Err(Failed)
            }
        }
    }
}


//------------ Passenger -----------------------------------------------------

data_enum! {
    pub enum Passenger {
        { None: "none" }
        { Restricted: "restricted" }
        { Historic: "historic" }
        { Seasonal: "seasonal" }
        { Tourist: "tourist" }
        { Full: "full" }
    }
}


//------------ Status --------------------------------------------------------

data_enum! {
    pub enum Status {
        { Planned: "planned" }
        { Construction: "construction" }
        { Open: "open" }
        { Suspended: "suspended" }
        { Reopened: "reopened" }
        { Closed: "closed" }
        { Removed: "removed" }
        { Released: "released" }
    }
}


//------------ DeVzg ---------------------------------------------------------

pub type DeVzg = Marked<String>;



//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="start attribute not allowed when sections is present")]
pub struct StartWithSections;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="end attribute not allowed when sections is present")]
pub struct EndWithSections;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid gauge (must be an integer followed by 'mm'")]
pub struct InvalidGauge;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid course segment")]
pub struct InvalidCourseSegment;

