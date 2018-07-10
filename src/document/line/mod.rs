
use std::str::FromStr;
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::store::{
    LoadStore, Stored, UpdateStore,
    LineLink, OrganizationLink, PathLink, PointLink, SourceLink
};
use ::types::{
    Date, EventDate, IntoMarked, Key, LanguageText, List, LocalText, Location,
    Marked, Set
};
use super::Point;
use super::common::{Alternative, Basis, Common, Contract, Progress};

mod verify;


//------------ Line ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Line {
    common: Common,
    label: Set<Label>,
    note: Option<LanguageText>,
    events: EventList,
    points: Points,
}

impl Line {
    pub fn common(&self) -> &Common {
        &self.common
    }

    pub fn key(&self) -> &Key {
        self.common().key()
    }

    pub fn origin(&self) -> &Origin {
        &self.common().origin()
    }
}

impl<'a> Stored<'a, Line> {
    pub fn common(&self) -> &Common {
        &self.access().common
    }

    pub fn key(&self) -> &Key {
        self.common().key()
    }

    pub fn progress(&self) -> Progress {
        self.common().progress()
    }

    pub fn origin(&self) -> &Origin {
        &self.common().origin()
    }

    pub fn label(&self) -> &Set<Label> {
        &self.access().label
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }

    pub fn events(&self) -> Stored<'a, EventList> {
        self.map(|item| &item.events)
    }

    pub fn points(&self) -> Stored<'a, Points> {
        self.map(|item| &item.points)
    }
}

impl Line {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &mut LoadStore,
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
        &mut self,
        link: LineLink,
        store: &mut UpdateStore,
        _report: &mut StageReporter
    ) {
        for point in self.points.iter() {
            point.update(store, |point| point.add_line(link.clone()))
        }
    }

    pub fn verify(&self, report: &mut StageReporter) {
        verify::verify(self, report)
    }
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

#[derive(Clone, Debug)]
pub struct Points {
    points: Vec<Marked<PointLink>>,
    indexes: Vec<(PointLink, usize)>,
}

impl Points {
    pub fn get_index(&self, link: &PointLink) -> Option<usize> {
        self.indexes.binary_search_by(|x| link.cmp(&x.0)).ok()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=PointLink> + 'a {
        self.points.iter().map(|link| link.as_value().clone())
    }
}

impl FromYaml<LoadStore> for Points {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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

#[derive(Clone, Debug)]
pub struct Event {
    date: EventDate,
    sections: List<Section>,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    alternative: List<Alternative>,
    basis: List<Basis>,
    note: Option<LanguageText>,

    concession: Option<Concession>,
    contract: Option<Contract>,
    treaty: Option<Contract>,

    category: Option<Set<Category>>,
    constructor: Option<List<Marked<OrganizationLink>>>,
    course: Option<List<CourseSegment>>,
    electrified: Option<Option<Set<Electrified>>>,
    freight: Option<Freight>,
    gauge: Option<Set<Gauge>>,
    local_name: Option<LocalText>, // XXX Drop
    name: Option<LocalText>,
    operator: Option<List<Marked<OrganizationLink>>>,
    owner: Option<List<Marked<OrganizationLink>>>,
    passenger: Option<Passenger>,
    rails: Option<Marked<u8>>,
    region: Option<List<Marked<OrganizationLink>>>,
    reused: Option<List<Marked<LineLink>>>,
    status: Option<Status>,
    tracks: Option<Marked<u8>>,

    de_vzg: Option<DeVzg>,
}

impl<'a> Stored<'a, Event> {
    pub fn date(&self) -> &EventDate {
        &self.access().date
    }

    pub fn sections(&self) -> Stored<'a, List<Section>> {
        self.map(|item| &item.sections)
    }

    pub fn document(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.document)
    }

    pub fn source(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.source)
    }

    pub fn alternative(&self) -> Stored<'a, List<Alternative>> {
        self.map(|item| &item.alternative)
    }

    pub fn basis(&self) -> Stored<'a, List<Basis>> {
        self.map(|item| &item.basis)
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }

    pub fn concession(&self) -> Option<Stored<'a, Concession>> {
        self.map_opt(|item| item.concession.as_ref())
    }

    pub fn contract(&self) -> Option<Stored<'a, Contract>> {
        self.map_opt(|item| item.contract.as_ref())
    }

    pub fn treaty(&self) -> Option<Stored<'a, Contract>> {
        self.map_opt(|item| item.treaty.as_ref())
    }

    pub fn category(&self) -> Option<&Set<Category>> {
        self.access().category.as_ref()
    }

    pub fn constructor(
        &self
    ) -> Option<Stored<'a, List<Marked<OrganizationLink>>>> {
        self.map_opt(|item| item.constructor.as_ref())
    }

    pub fn course(&self) -> Option<Stored<'a, List<CourseSegment>>> {
        self.map_opt(|item| item.course.as_ref())
    }

    pub fn electrified(&self) -> Option<Option<&Set<Electrified>>> {
        match self.access().electrified {
            Some(Some(ref some)) => Some(Some(some)),
            Some(None) => Some(None),
            None => None
        }
    }

    pub fn freight(&self) -> Option<Freight> {
        self.access().freight
    }

    pub fn gauge(&self) -> Option<&Set<Gauge>> {
        self.access().gauge.as_ref()
    }

    pub fn local_name(&self) -> Option<&LocalText> {
        self.access().local_name.as_ref()
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.access().name.as_ref()
    }

    pub fn operator(
        &self
    ) -> Option<Stored<'a, List<Marked<OrganizationLink>>>> {
        self.map_opt(|item| item.operator.as_ref())
    }

    pub fn owner(
        &self
    ) -> Option<Stored<'a, List<Marked<OrganizationLink>>>> {
        self.map_opt(|item| item.owner.as_ref())
    }

    pub fn passenger(&self) -> Option<Passenger> {
        self.access().passenger
    }

    pub fn rails(&self) -> Option<u8> {
        self.access().rails.map(Marked::into_value)
    }

    pub fn region(
        &self
    ) -> Option<Stored<'a, List<Marked<OrganizationLink>>>> {
        self.map_opt(|item| item.region.as_ref())
    }

    pub fn reused(&self) -> Option<Stored<'a, List<Marked<LineLink>>>> {
        self.map_opt(|item| item.reused.as_ref())
    }

    pub fn status(&self) -> Option<Status> {
        self.access().status
    }

    pub fn tracks(&self) -> Option<u8> {
        self.access().tracks.map(Marked::into_value)
    }

    pub fn de_vzg(&self) -> Option<&DeVzg> {
        self.access().de_vzg.as_ref()
    }
}

impl FromYaml<LoadStore> for Event {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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

#[derive(Clone, Debug)]
pub struct Section {
    start: Option<Marked<PointLink>>,
    end: Option<Marked<PointLink>>,
}

impl<'a> Stored<'a, Section> {
    pub fn start(&self) -> Option<&Point> {
        self.map_opt(|item| item.start.as_ref()).map(|x| x.follow())
    }

    pub fn end(&self) -> Option<&Point> {
        self.map_opt(|item| item.end.as_ref()).map(|x| x.follow())
    }
}


impl FromYaml<LoadStore> for Section {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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

#[derive(Clone, Debug)]
pub struct Concession {
    by: List<Marked<OrganizationLink>>,
    to: List<Marked<OrganizationLink>>,
    until: Option<Marked<Date>>,
}

impl<'a> Stored<'a, Concession> {
    pub fn by(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.by)
    }

    pub fn to(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.to)
    }

    pub fn until(&self) -> Option<&Marked<Date>> {
        self.access().until.as_ref()
    }
}

impl FromYaml<LoadStore> for Concession {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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

#[derive(Clone, Debug)]
pub struct CourseSegment {
    path: Marked<PathLink>,
    start: Marked<String>,
    end: Marked<String>,
}

impl<'a> Stored<'a, CourseSegment> {
    pub fn path(&self) -> Stored<'a, Marked<PathLink>> {
        self.map(|item| &item.path)
    }

    pub fn start(&self) -> &str {
        self.access().start.as_value().as_ref()
    }
    
    pub fn end(&self) -> &str {
        self.access().end.as_value().as_ref()
    }
}

impl FromYaml<LoadStore> for CourseSegment {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
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
        let path = PathLink::forge(key, context, report)?;
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Gauge(Marked<u16>);

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
        _: &mut C,
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

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="start attribute not allowed when sections is present")]
pub struct StartWithSections;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="end attribute not allowed when sections is present")]
pub struct EndWithSections;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid gauge (must be an integer followed by 'mm'")]
pub struct InvalidGauge;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid course segment")]
pub struct InvalidCourseSegment;

