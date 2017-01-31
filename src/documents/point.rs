use std::str;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, Sequence, ValueItem};
use super::document::DocumentType;


//------------ Point ---------------------------------------------------------

pub struct Point {
    key: String,
}

impl Point {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder) -> Result<Point, ()> {
        Ok(Point {
            key: key
        })
    }
}

impl Point {
    pub fn key(&self) -> &str {
        &self.key
    }
}

/*
impl PointDocument {
    pub fn parse(_item: Item<Scalar>, _path: &Path,
                 _collection: &CollectionBuilder,
                 _vars: &StackedMap<Value<Scalar>>,
                 _errors: &mut Vec<Error>)
                 -> Option<(String, Self)> {
        None
    }
}


use ::collection::{Key, Reference};
use super::{Line, Path, Source, Structure};
use super::common::{Date, LocalizedString, Progress}; 


pub enum Category {
    De(DeCategory),
    Dk(DkCategory),
    No(NoCategory),
}

pub enum DeCategory {
    Bf, Hp, Bft, Hst, Bk, Abzw, Dkst, Uest, Uehst,
    Awanst, Anst,
    Ldst, Ahst, Gnst, Ga,
    Stw, Po, Glgr,
    EGr, LGr, Strw,
    Tp, Gp, Ust,
    Museum
}

pub enum DkCategory {
    St, T, Smd,
    Gr
}

pub enum NoCategory {
    S, Sp, Hp
}


pub struct Location {
    pub line: Reference<Line>,
    pub location: String
}


pub enum Service {
    Full,
    None,
    Passenger,
    Freight
}
*/

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Side {
    Left, Right, Up, Down, Center
}

impl str::FromStr for Side {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Side::Left),
            "right" => Ok(Side::Right),
            "up" => Ok(Side::Up),
            "down" => Ok(Side::Down),
            "center" => Ok(Side::Center),
            _ => Err(format!("unknown side '{}'", s))
        }
    }
}

/*
pub struct Site {
    pub path: Reference<Path>,
    pub node: String,
    pub side: Side
}


pub enum Staff {
    Full,
    Agent,
    None
}


pub enum Status {
    Open,
    Suspended,
    Closed
}


pub enum Subtype {
    Border,
    Break,
    Portal,
    Post,
    Reference
}

impl Default for Subtype {
    fn default() -> Self {
        Subtype::Post
    }
}


pub enum DeRangklasse {
    I, II, III, IV, V, VI, U, S
}


pub struct Event {
    pub date: Option<Date>,
    pub sources: Vec<Reference<Source>>,

    pub category: Option<Vec<Category>>,
    pub connection: Option<Vec<Point>>,
    pub local_name: Option<LocalizedString>,
    pub location: Option<Vec<Location>>,
    pub master: Option<Reference<Point>>,
    pub merged: Option<Reference<Point>>,
    pub name: Option<String>,
    pub note: Option<LocalizedString>,
    pub public_name: Option<Vec<String>>,
    pub service: Option<Vec<Service>>,
    pub site: Option<Vec<Site>>,
    pub short_name: Option<String>,
    pub split_from: Option<Reference<Point>>,
    pub staff: Option<Staff>,
    pub status: Option<Status>,

    pub de: DeEvent,
    pub dk: DkEvent,
    pub no: NoEvent,
}

pub struct DeEvent {
    pub ds100: Option<String>,
    pub dstnr: Option<String>,
    pub lknr: Option<String>,
    pub rangklasse: Option<DeRangklasse>,
    pub vbl: Option<String>,
}

pub struct DkEvent {
    pub code: Option<String>,
}

pub struct NoEvent {
    pub fs: Option<String>,
    pub njk: Option<String>,
    pub nsb: Option<String>,
}


pub struct Point {
    pub key: Key,
    pub subtype: Subtype,
    pub progress: Progress,
    pub junction: bool,
    pub events: Vec<Event>,
    pub structure: Reference<Structure>,
}
*/


//------------ PointRef ------------------------------------------------------

pub struct PointRef(DocumentRef);

impl PointRef {
    pub fn get(&self) -> DocumentGuard<Point> {
        self.0.get()
    }
}

impl FromYaml for PointRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(PointRef(builder.ref_doc(item.value(), item.source(),
                                    DocumentType::Point)))
    }
}

