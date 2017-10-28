//! A line document.

use std::{fmt, ops};
use std::str::FromStr;
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::crosslink::CrosslinkContext;
use ::load::yaml::{Mapping, Value};
use ::links::{DocumentLink, LineLink, OrganizationLink, PathLink, PointLink,
              SourceLink};
use ::types::{Date, EventDate, Key, LanguageText, List, LocalText, Location,
              Marked, Set};
use super::common::{Alternative, Basis, Contract, Common};


//------------ Line ----------------------------------------------------------

pub struct Line {
    common: Common,
    label: Set<Label>,
    note: Option<LanguageText>,
    events: EventList,
    points: List<Marked<PointLink>>,
}

impl Line {
    pub fn label(&self) -> &Set<Label> {
        &self.label
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.note.as_ref()
    }

    pub fn events(&self) -> &EventList {
        &self.events
    }

    pub fn points(&self) -> &List<Marked<PointLink>> {
        &self.points
    }
}

impl Line {
    pub fn construct(key: Marked<Key>, mut doc: Marked<Mapping>,
                     context: &mut ConstructContext) -> Result<Self, Failed> {
        let common = Common::construct(key, &mut doc, context);
        let label = doc.take_default("label", context);
        let note = doc.take_opt("note", context);
        let events = doc.take("events", context);
        let points = doc.take("points", context);
        doc.exhausted(context)?;
        Ok(Line {
            common: common?,
            label: label?,
            note: note?,
            events: events?,
            points: points?,
        })
    }

    pub fn crosslink(&self, link: DocumentLink,
                     _context: &mut CrosslinkContext) {
        //self.events.crosslink(context);
        let link = link.convert();
        for (n, point) in self.points.iter().enumerate() {
            point.with_mut(|point| point.push_line(link.clone(), n))
                 .unwrap()
        }
    }
}


impl ops::Deref for Line {
    type Target = Common;

    fn deref(&self) -> &Common {
        &self.common
    }
}

impl ops::DerefMut for Line {
    fn deref_mut(&mut self) -> &mut Common {
        &mut self.common
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


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;

impl EventList {
    /*
    fn crosslink<C: Context>(&mut self, context: &mut C) {
        for event in self {
            event.crosslink(context)
        }
    }
    */
}

//------------ Event ---------------------------------------------------------

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
    course: List<CourseSegment>,
    electrified: Option<Set<Electrified>>,
    freight: Option<Freight>,
    gauge: Option<Set<Gauge>>,
    local_name: Option<LocalText>,
    name: Option<Marked<String>>,
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


impl Event {
    pub fn date(&self) -> &EventDate { &self.date } 
    pub fn sections(&self) -> &List<Section> { &self.sections }
    pub fn document(&self) -> &List<Marked<SourceLink>> { &self.document }
    pub fn source(&self) -> &List<Marked<SourceLink>> { &self.source }
    pub fn alternative(&self) -> &List<Alternative> { &self.alternative }
    pub fn basis(&self) -> &List<Basis> { &self.basis }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }

    pub fn concession(&self) -> Option<&Concession> { self.concession.as_ref() }
    pub fn contract(&self) -> Option<&Contract> { self.contract.as_ref() }
    pub fn treaty(&self) -> Option<&Contract> { self.treaty.as_ref() }

    pub fn category(&self) -> Option<&Set<Category>> { self.category.as_ref() }
    pub fn constructor(&self) -> Option<&List<Marked<OrganizationLink>>> {
        self.constructor.as_ref()
    }
    pub fn course(&self) -> &List<CourseSegment> { &self.course }
    pub fn electrified(&self) -> Option<&Set<Electrified>> {
        self.electrified.as_ref()
    }
    pub fn freight(&self) -> Option<Freight> { self.freight }
    pub fn gauge(&self) -> Option<&Set<Gauge>> { self.gauge.as_ref() }
    pub fn local_name(&self) -> Option<&LocalText> { self.local_name.as_ref() }
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|v| v.as_value().as_ref())
    }
    pub fn operator(&self) -> Option<&List<Marked<OrganizationLink>>> {
        self.operator.as_ref()
    }
    pub fn owner(&self) -> Option<&List<Marked<OrganizationLink>>> {
        self.owner.as_ref()
    }
    pub fn passenger(&self) -> Option<Passenger> { self.passenger }
    pub fn rails(&self) -> Option<u8> {
        self.rails.as_ref().map(Marked::to_value)
    }
    pub fn region(&self) -> Option<&List<Marked<OrganizationLink>>> {
        self.region.as_ref()
    }
    pub fn reused(&self) -> Option<&List<Marked<LineLink>>> {
        self.reused.as_ref()
    }
    pub fn status(&self) -> Option<Status> { self.status }
    pub fn tracks(&self) -> Option<u8> {
        self.tracks.as_ref().map(Marked::to_value)
    }

    pub fn de_vzg(&self) -> Option<&DeVzg> { self.de_vzg.as_ref() }
}

impl Constructable for Event {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let date = value.take("date", context);
        let sections = value.take_default("sections", context);
        let start = value.take_opt("start", context);
        let end = value.take_opt("end", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        let alternative = value.take_default("alternative", context);
        let basis = value.take_default("basis", context);
        let note = value.take_opt("note", context);

        let concession = value.take_opt("concession", context);
        let contract = value.take_opt("contract", context);
        let treaty = value.take_opt("treaty", context);
        
        let category = value.take_opt("category", context);
        let constructor = value.take_opt("constructor", context);
        let course = value.take_default("course", context);
        let electrified = value.take_opt("electrified", context);
        let freight = value.take_opt("freight", context);
        let gauge = value.take_opt("gauge", context);
        let local_name = value.take_opt("local_name", context);
        let name = value.take_opt("name", context);
        let operator = value.take_opt("operator", context);
        let owner = value.take_opt("owner", context);
        let passenger = value.take_opt("passenger", context);
        let rails = value.take_opt("rails", context);
        let region = value.take_opt("region", context);
        let reused = value.take_opt("reused", context);
        let status = value.take_opt("status", context);
        let tracks = value.take_opt("tracks", context);

        let de_vzg = value.take_opt("de.VzG", context);

        value.exhausted(context)?;

        let mut sections: List<Section> = sections?;
        let start: Option<Marked<PointLink>> = start?;
        let end: Option<Marked<PointLink>> = end?;
        match (start, end) {
            (None, None) => { },
            (start, end) => {
                if !sections.is_empty() {
                    if let Some(start) = start {
                        context.push_error((StartWithSections,
                                            start.location()));
                    }
                    if let Some(end) = end {
                        context.push_error((EndWithSections, end.location()));
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

impl Event {
    /*
    fn crosslink<C: Context>(&mut self, _context: &mut C) {
    }
    */
}


//------------ Section -------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Section {
    start: Option<Marked<PointLink>>,
    end: Option<Marked<PointLink>>,
}

impl Section {
    pub fn start(&self) -> Option<&Marked<PointLink>> {
        self.start.as_ref()
    }

    pub fn end(&self) -> Option<&Marked<PointLink>> {
        self.end.as_ref()
    }
}


impl Constructable for Section {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let start = value.take_opt("start", context);
        let end = value.take_opt("end", context);
        value.exhausted(context)?;
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

impl Concession {
    pub fn by(&self) -> &List<Marked<OrganizationLink>> { &self.by }
    pub fn to(&self) -> &List<Marked<OrganizationLink>> { &self.to }
    pub fn until(&self) -> Option<&Marked<Date>> { self.until.as_ref() }
}

impl Constructable for Concession {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let by = value.take_default("by", context);
        let to = value.take_default("for", context);
        let until = value.take_opt("until", context);
        value.exhausted(context)?;
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

impl CourseSegment {
    pub fn path(&self) -> &Marked<PathLink> { &self.path }
    pub fn start(&self) -> &str { self.start.as_value().as_ref() }
    pub fn end(&self) -> &str { self.end.as_value().as_ref() }
}

impl Constructable for CourseSegment {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let (value, location) = value.into_string(context)?.unwrap();
        let mut value = value.split_whitespace();
        let path = match value.next() {
            Some(path) => path,
            None => {
                context.push_error((InvalidCourseSegment, location));
                return Err(Failed)
            }
        };
        let key = context.ok(Key::from_str(path)
                                 .map_err(|err| (err, location)))?;
        let path = Marked::new(PathLink::from_key(key, location, context)?,
                               location);
        let start = match value.next() {
            Some(path) => path,
            None => {
                context.push_error((InvalidCourseSegment, location));
                return Err(Failed)
            }
        };
        let start = Marked::new(String::from(start), location);
        let end = match value.next() {
            Some(path) => path,
            None => {
                context.push_error((InvalidCourseSegment, location));
                return Err(Failed)
            }
        };
        let end = Marked::new(String::from(end), location);
        if value.next().is_some() {
            context.push_error((InvalidCourseSegment, location));
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

impl Constructable for Gauge {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let (value, location) = value.into_string(context)?.unwrap();
        if !value.ends_with("mm") {
            context.push_error((InvalidGauge, location));
            return Err(Failed)
        }
        match u16::from_str(&value[0..value.len() - 2]) {
            Ok(value) => Ok(Gauge(Marked::new(value, location))),
            Err(_) => {
                context.push_error((InvalidGauge, location));
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

#[derive(Clone, Copy, Debug)]
pub struct StartWithSections;

impl fmt::Display for StartWithSections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("start attribute not allowed when sections is present")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EndWithSections;

impl fmt::Display for EndWithSections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("end attribute not allowed when sections is present")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidGauge;

impl fmt::Display for InvalidGauge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid gauge (must be an integer followed by 'mm'")
    }
}


#[derive(Clone, Copy, Debug)]
pub struct InvalidCourseSegment;

impl fmt::Display for InvalidCourseSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid course segment")
    }
}

