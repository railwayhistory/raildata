use std::{fmt, ops};
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::crosslink::CrosslinkContext;
use ::load::yaml::{Mapping, Value};
use ::links::{DocumentLink, LineLink, PathLink, PointLink, SourceLink};
use ::types::{EventDate, Key, LanguageText, List, LocalText, Marked, Set};
use super::common::Common;


//------------ Point ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Point {
    // Attributes
    common: Common,
    events: List<Event>,
    junction: Option<Marked<bool>>,
    subtype: Marked<Subtype>,

    // Cross-links
    lines: List<(LineLink, usize)>,
}

impl Point {
    pub fn events(&self) -> &List<Event> { &self.events }
    pub fn junction(&self) -> Option<bool> {
        self.junction.as_ref().map(Marked::to_value)
    }
    pub fn subtype(&self) -> Marked<Subtype> { self.subtype }

    pub fn lines(&self) -> &List<(LineLink, usize)> { &self.lines }
}

impl Point {
    pub fn construct(key: Marked<Key>, mut doc: Marked<Mapping>,
                     context: &mut ConstructContext) -> Result<Self, Failed> {
        let common = Common::construct(key, &mut doc, context);
        let events = doc.take("events", context);
        let junction = doc.take_opt("junction", context);
        let subtype = doc.take_default("subtype", context);
        doc.exhausted(context)?;
        Ok(Point {
            common: common?,
            events: events?,
            junction: junction?,
            subtype: subtype?,

            lines: List::default(),
        })
    }

    pub fn crosslink(&self, _link: DocumentLink,
                     _context: &mut CrosslinkContext) {
    }
}

impl Point {
    pub fn push_line(&mut self, link: LineLink, n: usize) {
        self.lines.push((link, n))
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
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    note: Option<LanguageText>,

    category: Option<Set<Category>>,
    connection: Option<List<Marked<PointLink>>>,
    designation: Option<Marked<String>>,
    location: Option<Location>,
    master: Option<Option<Marked<PointLink>>>,
    merged: Option<Marked<PointLink>>,
    name: Option<LocalText>,
    plc: Option<Plc>,
    public_name: Option<List<LocalText>>,
    site: Option<List<Site>>,
    short_name: Option<LocalText>,
    staff: Option<Staff>,
    status: Option<Status>,

    service: Option<Service>,
    split_from: Option<Marked<PointLink>>,

    de_ds100: Option<DeDs100>,
    de_dstnr: Option<DeDstnr>,
    de_lknr: Option<List<DeLknr>>,
    de_name16: Option<DeName16>,
    de_rang: Option<DeRang>,
    de_vbl: Option<DeVbl>,

    dk_ref: Option<Marked<String>>,

    no_fs: Option<Marked<String>>,
    no_njk: Option<Marked<String>>,
    no_nsb: Option<Marked<String>>,
}

impl Event {
    pub fn date(&self) -> &EventDate { &self.date }
    pub fn document(&self) -> &List<Marked<SourceLink>> { &self.document }
    pub fn source(&self) -> &List<Marked<SourceLink>> { &self.source }
    pub fn note(&self) -> Option<&LanguageText> { self.note.as_ref() }

    pub fn category(&self) -> Option<&Set<Category>> {
        self.category.as_ref()
    }
    pub fn connection(&self) -> Option<&List<Marked<PointLink>>> {
        self.connection.as_ref()
    }
    pub fn designation(&self) -> Option<&str> {
        self.designation.as_ref().map(AsRef::as_ref)
    }
    pub fn location(&self) -> Option<&Location> { self.location.as_ref() }
    pub fn master(&self) -> Option<Option<&Marked<PointLink>>> {
        self.master.as_ref().map(Option::as_ref)
    }
    pub fn merged(&self) -> Option<&Marked<PointLink>> { self.merged.as_ref() }
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
    pub fn split_from(&self) -> Option<&Marked<PointLink>> {
        self.split_from.as_ref()
    }

    pub fn de_ds100(&self) -> Option<&DeDs100> { self.de_ds100.as_ref() }
    pub fn de_dstnr(&self) -> Option<&DeDstnr> { self.de_dstnr.as_ref() }
    pub fn de_lknr(&self) -> Option<&List<DeLknr>> { self.de_lknr.as_ref() }
    pub fn de_name16(&self) -> Option<&DeName16> { self.de_name16.as_ref() }
    pub fn de_rang(&self) -> Option<DeRang> { self.de_rang }
    pub fn de_vbl(&self) -> Option<&DeVbl> { self.de_vbl.as_ref() }

    pub fn dk_ref(&self) -> Option<&str> {
        self.dk_ref.as_ref().map(AsRef::as_ref)
    }

    pub fn no_fs(&self) -> Option<&str> {
        self.no_fs.as_ref().map(AsRef::as_ref)
    }
    pub fn no_njk(&self) -> Option<&str> {
        self.no_njk.as_ref().map(AsRef::as_ref)
    }
    pub fn no_nsb(&self) -> Option<&str> {
        self.no_nsb.as_ref().map(AsRef::as_ref)
    }
}

impl Constructable for Event {
    fn construct(value: Value, context: &mut ConstructContext)
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
pub struct Location(List<(Marked<LineLink>, Option<Marked<String>>)>);

impl Constructable for Location {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut list = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(context)? {
            let key = match Marked::<LineLink>::from_string(key, context) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue
                }
            };
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

pub type Plc = Marked<String>;


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
    path: Marked<PathLink>,
    node: Marked<String>,
}

impl Site {
    pub fn path(&self) -> &Marked<PathLink> { &self.path }
    pub fn node(&self) -> &str { self.node.as_value().as_ref() }
}

impl Constructable for Site {
    fn construct(value: Value, context: &mut ConstructContext)
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
        let path = Marked::new(String::from(path), location);
        let path = Marked::<PathLink>::from_string(path, context)?;
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

pub type DeDs100 = Marked<String>;


//------------ DeDstnr -------------------------------------------------------

pub type DeDstnr = Marked<String>;


//------------ DeLknr --------------------------------------------------------

pub type DeLknr = Marked<String>; 


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

pub type DeVbl = Marked<String>;


//------------ DeName16 ------------------------------------------------------

pub type DeName16 = Marked<String>;


//============ Errors ========================================================

#[derive(Clone, Copy, Debug)]
pub struct InvalidSite;

impl fmt::Display for InvalidSite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid site value")
    }
}

