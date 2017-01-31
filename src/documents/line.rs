use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, Sequence, ValueItem};
use super::common::{Progress, ShortVec};
use super::date::Date;
use super::document::DocumentType;
use super::point::{Point, PointRef};


//------------ Line ----------------------------------------------------------

pub struct Line {
    key: String,
    progress: Progress,
    label: Option<Label>,
    events: Events,
    points: ShortVec<PointRef>,
}

impl Line {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder) -> Result<Line, ()> {
        let progress = Progress::from_yaml(item.optional_key("progress"),
                                           builder);
        let label = item.parse_opt("label", builder);
        let events = Events::from_yaml(item.mandatory_key("events", builder)?
                                           .into_sequence(builder)?,
                                       builder);
        let points = item.parse("points", builder);
        Ok(Line {
            key: key,
            progress: progress?,
            label: label?,
            events: events?,
            points: points?,
        })
    }
}

impl Line {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn label(&self) -> Option<Label> {
        self.label
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn points(&self) -> &ShortVec<PointRef> {
        &self.points
    }
}


//------------ Label ---------------------------------------------------------

mandatory_enum! {
    pub enum Label {
        (Connection => "connection"),
        (Freight => "freight"),
        (Port => "port"),
        (DeSBahn => "de.S-Bahn"),
    }
}


//------------ Events --------------------------------------------------------

pub struct Events(Vec<Event>);

impl Events {
    fn from_yaml(item: Sequence, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut res = Some(Vec::new());
        for event in item {
            if let Ok(event) = Event::from_yaml(event, builder) {
                if let Some(ref mut res) = res {
                    res.push(event)
                }
            }
        }
        res.ok_or(()).map(Events)
    }
}

impl ops::Deref for Events {
    type Target = [Event];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


//------------ Event ---------------------------------------------------------

pub struct Event {
    date: Option<Date>,
    //sections: ShortVec<Section>,
}

impl Event {
    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }
}

impl Event {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let date = item.parse_opt("date", builder);
        //let sections = Section::from_yaml(item, builder);
        Ok(Event {
            date: date?,
            //sections: sections?,
        })
    }
}


/*
//------------ Section -------------------------------------------------------

pub struct Section(Option<PointRef>, Option<PointRef>);

impl Section {
    pub fn start(&self) -> Option<DocumentGuard<Point>> {
        self.0.as_ref().map(PointRef::get)
    }

    pub fn end(&self) -> Option<DocumentGuard<Point>> {
        self.1.as_ref().map(PointRef::get)
    }
}

impl FromYaml for Section {
    fn from_yaml(event: &mut Item<Mapping>,
                 collection: &CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<ShortVec<Self>, ()> {
        let start = event.parse_opt::<String>("start", collection, errors);
        let end = event.parse_opt::<String>("end", collection, errors);
        let sections = event.parse_opt::<


use ::collection::{Key, Reference};
use super::common::{Date, LocalizedString, Progress}; 
use super::organization::Organization;
use super::path::Path;
use super::point::Point;
use super::source::Source;


pub enum Category {
    De(DeCategory),
}

pub enum DeCategory {
    Hauptbahn,
    Nebenbahn,
    Kleinbahn,
    Anschl,
    Bfgleis,
    Strab
}

pub struct Concession {
    pub by: Vec<Reference<Organization>>,
    pub to: Vec<Reference<Organization>>,
    pub until: Option<Date>,
    pub document: Vec<Reference<Source>>,
}

pub struct Contract {
    pub parties: Vec<Reference<Organization>>,
    pub document: Vec<Reference<Source>>,
}

pub struct CourseItem {
    pub path: Reference<Path>,
    pub start: String,
    pub end: String,
    pub offset: f64,
}

*/

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Voltage {
    Ac {
        volts: u32,
        frequency: u8,
        phases: u8,
    },
    Dc {
        volts: u32,
    },
}

/*

pub enum ContactRail {
    Top,
    Inside,
    Outside,
    Bottom
}

pub struct ElectricRail {
    pub voltage: Voltage,
    pub rails: u8,
    pub contact: ContactRail
}

pub struct Electrified {
    pub rail: Option<ElectricRail>,
    pub overhead: Option<Voltage>,
}

pub enum Freight {
    None,
    Restricted,
    Full
}

pub struct Gauge(pub u32);

pub enum Passenger {
    None,
    Restricted,
    Historic,
    Seasonal,
    Tourist,
    Full
}

pub enum Status {
    Planned,
    Construction,
    Open,
    Suspended,
    Reopened,
    Closed,
    Removed,
    Released,
}

pub struct Event {
    pub sections: Vec<(Reference<Point>, Reference<Point>)>,
    pub sources: Vec<Reference<Source>>,

    pub category: Option<Vec<Category>>,
    pub concession: Option<Concession>,
    pub contract: Option<Contract>,
    pub course: Option<Vec<CourseItem>>,
    pub electrified: Option<Electrified>,
    pub freight: Option<Freight>,
    pub gauge: Option<Vec<Gauge>>,
    pub local_name: Option<LocalizedString>,
    pub name: Option<String>,
    pub note: Option<LocalizedString>,
    pub operator: Vec<Reference<Organization>>,
    pub owner: Vec<Reference<Organization>>,
    pub passenger: Option<Passenger>,
    pub rails: Option<u8>,
    pub region: Option<Vec<Reference<Organization>>>,
    pub reused: Option<Vec<Reference<Line>>>,
    pub status: Option<Status>,
    pub tracks: Option<u8>,
    pub treaty: Option<Contract>,

    pub de: Option<DeEvent>,
}

pub struct DeEvent {
    pub vzg: Option<String>
}
*/


//------------ LineRef -------------------------------------------------------

pub struct LineRef(DocumentRef);

impl LineRef {
    pub fn get(&self) -> DocumentGuard<Line> {
        self.0.get()
    }
}

impl FromYaml for LineRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(LineRef(builder.ref_doc(item.value(), item.source(),
                                   DocumentType::Line)))
    }
}

