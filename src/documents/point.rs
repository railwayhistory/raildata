use std::{fmt, ops};
use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::{MarkedMapping, Value};
use super::common::Common;
use super::links::{LineLink, PathLink, PointLink, SourceLink};
use super::types::{Boolean, EventDate, LanguageText, List, LocalText,
                   Marked, Set, Text};


//------------ Point ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Point {
    common: Common,
    events: List<Event>,
    junction: Option<Boolean>,
    subtype: Marked<Subtype>,
}

impl Point {
    pub fn events(&self) -> &List<Event> { &self.events }
    pub fn junction(&self) -> Option<Boolean> { self.junction }
    pub fn subtype(&self) -> Marked<Subtype> { self.subtype }
}

impl Point {
    pub fn construct<C: Context>(common: Common, mut doc: MarkedMapping,
                                 context: &mut C) -> Result<Self, Failed> {
        let events = doc.take("events", context);
        let junction = doc.take_opt("junction", context);
        let subtype = doc.take_default("subtype", context);
        doc.exhausted(context)?;
        Ok(Point { common,
            events: events?,
            junction: junction?,
            subtype: subtype?
        })
    }
}


impl ops::Deref for Point {
    type Target = Common;

    fn deref(&self) -> &Common {
        &self.common
    }
}

impl ops::DerefMut for Point {
    fn deref_mut(&mut self) -> &mut Common {
        &mut self.common
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


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Event {
    date: EventDate,
    document: List<SourceLink>,
    source: List<SourceLink>,
    note: Option<LanguageText>,

    category: Option<Set<Category>>,
    connection: Option<List<PointLink>>,
    designation: Option<Text>,
    location: Option<Location>,
    master: Option<Option<PointLink>>,
    merged: Option<PointLink>,
    name: Option<LocalText>,
    plc: Option<Plc>,
    public_name: Option<List<LocalText>>,
    site: Option<List<Site>>,
    short_name: Option<LocalText>,
    staff: Option<Staff>,
    status: Option<Status>,

    service: Option<Service>,
    split_from: Option<PointLink>,

    de_ds100: Option<DeDs100>,
    de_dstnr: Option<DeDstnr>,
    de_lknr: Option<List<DeLknr>>,
    de_name16: Option<DeName16>,
    de_rang: Option<DeRang>,
    de_vbl: Option<DeVbl>,

    dk_ref: Option<Text>,

    no_fs: Option<Text>,
    no_njk: Option<Text>,
    no_nsb: Option<Text>,
}

impl Event {
    pub fn date(&self) -> &EventDate { &self.date }
    pub fn document(&self) -> &List<SourceLink> { &self.document }
    pub fn source(&self) -> &List<SourceLink> { &self.source }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }

    pub fn category(&self) -> Option<&Set<Category>> {
        self.category.as_ref()
    }
    pub fn connection(&self) -> Option<&List<PointLink>> {
        self.connection.as_ref()
    }
    pub fn designation(&self) -> Option<&Text> { self.designation.as_ref() }
    pub fn location(&self) -> Option<&Location> { self.location.as_ref() }
    pub fn master(&self) -> Option<Option<&PointLink>> {
        self.master.as_ref().map(Option::as_ref)
    }
    pub fn merged(&self) -> Option<&PointLink> { self.merged.as_ref() }
    pub fn name(&self) -> Option<&LocalText> { self.name.as_ref() }
    pub fn plc(&self) -> Option<&Plc> { self.plc.as_ref() }
    pub fn public_name(&self) -> Option<&List<LocalText>> {
        self.public_name.as_ref()
    }
    pub fn site(&self) -> Option<&List<Site>> { self.site.as_ref() }
    pub fn short_name(&self) -> Option<&LocalText> {
        self.short_name.as_ref()
    }
    pub fn staff(&self) -> Option<Staff> { self.staff }
    pub fn status(&self) -> Option<Status> { self.status }

    pub fn service(&self) -> Option<Service> { self.service }
    pub fn split_from(&self) -> Option<&PointLink> {
        self.split_from.as_ref()
    }

    pub fn de_ds100(&self) -> Option<&DeDs100> { self.de_ds100.as_ref() }
    pub fn de_dstnr(&self) -> Option<&DeDstnr> { self.de_dstnr.as_ref() }
    pub fn de_lknr(&self) -> Option<&List<DeLknr>> { self.de_lknr.as_ref() }
    pub fn de_name16(&self) -> Option<&DeName16> { self.de_name16.as_ref() }
    pub fn de_rang(&self) -> Option<DeRang> { self.de_rang }
    pub fn de_vbl(&self) -> Option<&DeVbl> { self.de_vbl.as_ref() }

    pub fn dk_ref(&self) -> Option<&Text> { self.dk_ref.as_ref() }

    pub fn no_fs(&self) -> Option<&Text> { self.no_fs.as_ref() }
    pub fn no_njk(&self) -> Option<&Text> { self.no_njk.as_ref() }
    pub fn no_nsb(&self) -> Option<&Text> { self.no_nsb.as_ref() }
}

impl Constructable for Event {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;

        let date = value.take("date", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        let note = value.take_opt("note", context);

        let category = value.take_opt("category", context);
        let connection = value.take_opt("connection", context);
        let designation = value.take_opt("designation", context);
        let location = value.take_opt("location", context);
        let master = value.take_opt("master", context);
        let merged = value.take_opt("merged", context);
        let name = value.take_opt("name", context);
        let plc = value.take_opt("PLC", context);
        let public_name = value.take_opt("public_name", context);
        let site = value.take_opt("site", context);
        let short_name = value.take_opt("short_name", context);
        let staff = value.take_opt("staff", context);
        let status = value.take_opt("status", context);

        let service = value.take_opt("service", context);
        let split_from = value.take_opt("split_from", context);

        let de_ds100 = value.take_opt("de.DS100", context);
        let de_dstnr = value.take_opt("de.dstnr", context);
        let de_lknr = value.take_opt("de.lknr", context);
        let de_name16 = value.take_opt("de.name16", context);
        let de_rang = value.take_opt("de.rang", context);
        let de_vbl = value.take_opt("de.VBL", context);

        let dk_ref = value.take_opt("dk.ref", context);
        
        let no_fs = value.take_opt("no.fs", context);
        let no_njk = value.take_opt("no.NJK", context);
        let no_nsb = value.take_opt("no.NSB", context);

        value.exhausted(context)?;
        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            note: note?,
            category: category?,
            connection: connection?,
            designation: designation?,
            location: location?,
            master: master?,
            merged: merged?,
            name: name?,
            plc: plc?,
            public_name: public_name?,
            site: site?,
            short_name: short_name?,
            staff: staff?,
            status: status?,
            service: service?,
            split_from: split_from?,
            de_ds100: de_ds100?,
            de_dstnr: de_dstnr?,
            de_lknr: de_lknr?,
            de_name16: de_name16?,
            de_rang: de_rang?,
            de_vbl: de_vbl?,
            dk_ref: dk_ref?,
            no_fs: no_fs?,
            no_njk: no_njk?,
            no_nsb: no_nsb?,
        })
    }
}


//------------ Category ------------------------------------------------------

data_enum! {
    pub enum Category {
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

        { DkSt: "dk.St" }
        { DkT: "dk.T" }
        { DkSmd: "dk.Smd" }
        { DkGr: "dk.Gr" }

        { NoS: "no.s" }
        { NoSp: "no.sp" }
        { NoHp: "no.hp" }
    }
}


//------------ Location ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Location(List<(LineLink, Option<Text>)>);

impl Constructable for Location {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut list = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(context)? {
            let key = key.map(|key| context.get_link(&key.into()));
            if value.is_null() {
                list.push((key, None))
            }
            else if let Ok(value) = value.into_string(context) {
                list.push((key, Some(value)));
            }
            else {
                err = true;
            }
        }
        if err {
            Err(Failed)
        }
        else {
            Ok(Location(list))
        }
    }
}


//------------ Plc -----------------------------------------------------------

pub type Plc = Text;


//------------ Service -------------------------------------------------------

data_enum! {
    pub enum Service {
        { Full: "full" }
        { None: "none" }
        { Passenger: "passenger" }
        { Freight: "freight" }
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

#[derive(Clone, Debug)]
pub struct Site {
    path: PathLink,
    node: Text,
}

impl Site {
    pub fn path(&self) -> &PathLink { &self.path }
    pub fn node(&self) -> &Text { &self.node }
}

impl Constructable for Site {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let (value, location) = value.into_string(context)?.unwrap();
        let mut value = value.split_whitespace();
        let path = match value.next() {
            Some(path) => path,
            None => {
                context.push_error((InvalidSite, location));
                return Err(Failed)
            }
        };
        let path = Marked::new(String::from(path), location).into();
        let path = Marked::new(context.get_link(&path), location);
        let node = match value.next() {
            Some(path) => path,
            None => {
                context.push_error((InvalidSite, location));
                return Err(Failed)
            }
        };
        let node = Marked::new(String::from(node), location);
        if value.next().is_some() {
            context.push_error((InvalidSite, location));
            return Err(Failed)
        }
        Ok(Site { path, node })
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
        { Open: "open" }
        { Suspended: "suspended" }
        { Closed: "closed" }
        { Merged: "merged" }
    }
}


//------------ DeDs100 -------------------------------------------------------

pub type DeDs100 = Text;


//------------ DeDstnr -------------------------------------------------------

pub type DeDstnr = Text;


//------------ DeLknr --------------------------------------------------------

pub type DeLknr = Text; 


//------------ DeRang --------------------------------------------------------

data_enum! {
    pub enum DeRang {
        { I: "I" }
        { Ii: "II" }
        { Iii: "III" }
        { Iv: "IV" }
        { V: "V" }
        { Vi: "VI" }
        { U: "U" }
        { S: "S" }
    }
}


//------------ DeVbl ---------------------------------------------------------

pub type DeVbl = Text;


//------------ DeName16 ------------------------------------------------------

pub type DeName16 = Text;


//============ Errors ========================================================

#[derive(Clone, Copy, Debug)]
pub struct InvalidSite;

impl fmt::Display for InvalidSite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid site value")
    }
}

