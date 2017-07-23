use std::{ops, str};
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::error::Source;
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, Progress, ShortVec, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};
use super::line::{Line, LineRef};
use super::path::{Path, PathRef};


//------------ Point ---------------------------------------------------------

pub struct Point {
    key: String,
    subtype: Subtype,
    progress: Progress,
    junction: Option<bool>,
    events: Events,
}

impl Point {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn junction(&self) -> Option<bool> {
        self.junction
    }

    pub fn events(&self) -> &Events {
        &self.events
    }
}

impl Point {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let subtype = item.parse_default("subtype", builder);
        let progress = item.parse_default("progress", builder);
        let junction = item.parse_opt("junction", builder);
        let events = item.parse_mandatory("events", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Document::Point(Point {
            subtype: try_key!(subtype, key),
            progress: try_key!(progress, key),
            junction: try_key!(junction, key),
            events: try_key!(events, key),
            key: key,
        }))
    }
}


//------------ Subtype -------------------------------------------------------

optional_enum! {
    pub enum Subtype {
        (Border => "border"),
        (Break => "break"),
        (Post => "post"),
        (Reference => "reference"),

        default Post
    }
}


//------------ Events --------------------------------------------------------

pub struct Events(Vec<Event>);

impl FromYaml for Events {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_sequence(builder)?;
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
    sources: Sources,

    category: Option<ShortVec<Category>>,
    connection: Option<ShortVec<PointRef>>,
    local_name: Option<LocalizedString>,
    location: Option<ShortVec<Location>>,
    master: Option<PointRef>,
    merged: Option<PointRef>,
    name: Option<String>,
    note: Option<LocalizedString>,
    plc: Option<String>,
    public_name: Option<ShortVec<String>>,
    service: Option<Service>,
    site: Option<ShortVec<Site>>,
    short_name: Option<String>,
    split_from: Option<PointRef>,
    staff: Option<Staff>,
    status: Option<Status>,

    de_ds100: Option<String>,
    de_dstnr: Option<String>,
    de_lknr: Option<ShortVec<String>>,
    de_rang: Option<DeRangklasse>,
    de_vbl: Option<String>,
    de_name16: Option<String>,

    dk_ref: Option<String>,

    no_fs: Option<String>,
    no_njk: Option<String>,
    no_nsb: Option<String>,
}

impl Event {
    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn sources(&self) -> &Sources {
        &self.sources
    }

    pub fn category(&self) -> Option<&ShortVec<Category>> {
        self.category.as_ref()
    }

    pub fn connection(&self) -> Option<&ShortVec<PointRef>> {
        self.connection.as_ref()
    }

    pub fn local_name(&self) -> Option<&LocalizedString> {
        self.local_name.as_ref()
    }

    pub fn location(&self) -> Option<&ShortVec<Location>> {
        self.location.as_ref()
    }

    pub fn master(&self) -> Option<DocumentGuard<Point>> {
        self.master.as_ref().map(PointRef::get)
    }

    pub fn merged(&self) -> Option<DocumentGuard<Point>> {
        self.merged.as_ref().map(PointRef::get)
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn plc(&self) -> Option<&str> {
        self.plc.as_ref().map(AsRef::as_ref)
    }

    pub fn public_name(&self) -> Option<&ShortVec<String>> {
        self.public_name.as_ref()
    }

    pub fn service(&self) -> Option<Service> {
        self.service
    }

    pub fn site(&self) -> Option<&ShortVec<Site>> {
        self.site.as_ref()
    }

    pub fn short_name(&self) -> Option<&str> {
        self.short_name.as_ref().map(AsRef::as_ref)
    }

    pub fn split_from(&self) -> Option<DocumentGuard<Point>> {
        self.split_from.as_ref().map(PointRef::get)
    }

    pub fn staff(&self) -> Option<Staff> {
        self.staff
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    pub fn de_ds100(&self) -> Option<&str> {
        self.de_ds100.as_ref().map(AsRef::as_ref)
    }

    pub fn de_dstnr(&self) -> Option<&str> {
        self.de_dstnr.as_ref().map(AsRef::as_ref)
    }

    pub fn de_lknr(&self) -> Option<&ShortVec<String>> {
        self.de_lknr.as_ref()
    }

    pub fn de_rang(&self) -> Option<DeRangklasse> {
        self.de_rang
    }

    pub fn de_vbl(&self) -> Option<&str> {
        self.de_vbl.as_ref().map(AsRef::as_ref)
    }

    pub fn de_name16(&self) -> Option<&str> {
        self.de_name16.as_ref().map(AsRef::as_ref)
    }

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

impl FromYaml for Event {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let date = item.parse_opt("date", builder);
        let sources = Sources::from_opt_yaml(item.optional_key("sources"),
                                             builder);
        let category = item.parse_opt("category", builder);
        let connection = item.parse_opt("connection", builder);
        let local_name = item.parse_opt("local_name", builder);
        let location = item.parse_opt("location", builder);
        let master = item.parse_opt("master", builder);
        let merged = item.parse_opt("merged", builder);
        let name = item.parse_opt("name", builder);
        let note = item.parse_opt("note", builder);
        let plc = item.parse_opt("PLC", builder);
        let public_name = item.parse_opt("public_name", builder);
        let service = item.parse_opt("service", builder);
        let site = item.parse_opt("site", builder);
        let short_name = item.parse_opt("short_name", builder);
        let split_from = item.parse_opt("split_from", builder);
        let staff = item.parse_opt("staff", builder);
        let status = item.parse_opt("status", builder);
        let de_ds100 = item.parse_opt("de.DS100", builder);
        let de_dstnr = item.parse_opt("de.dstnr", builder);
        let de_lknr = item.parse_opt("de.lknr", builder);
        let de_rang = item.parse_opt("de.Rangklasse", builder);
        let de_vbl = item.parse_opt("de.VBL", builder);
        let de_name16 = item.parse_opt("de.name16", builder);
        let dk_ref = item.parse_opt("dk.ref", builder);
        let no_fs = item.parse_opt("no.fs", builder);
        let no_njk = item.parse_opt("no.NJK", builder);
        let no_nsb = item.parse_opt("no.NSB", builder);
        item.exhausted(builder)?;

        Ok(Event {
            date: date?,
            sources: sources?,
            category: category?,
            connection: connection?,
            local_name: local_name?,
            location: location?,
            master: master?,
            merged: merged?,
            name: name?,
            note: note?,
            plc: plc?,
            public_name: public_name?,
            service: service?,
            site: site?,
            short_name: short_name?,
            split_from: split_from?,
            staff: staff?,
            status: status?,
            de_ds100: de_ds100?,
            de_dstnr: de_dstnr?,
            de_lknr: de_lknr?,
            de_rang: de_rang?,
            de_vbl: de_vbl?,
            de_name16: de_name16?,
            dk_ref: dk_ref?,
            no_fs: no_fs?,
            no_njk: no_njk?,
            no_nsb: no_nsb?,
        })
    }
}


//------------ Category ------------------------------------------------------

mandatory_enum! {
    pub enum Category {
        (DeBf => "de.Bf"),
        (DeHp => "de.Hp"),
        (DeBft => "de.Bft"),
        (DeHst => "de.Hst"),
        (DeKr => "de.Kr"),
        (DeBk => "de.Bk"),
        (DeAbzw => "de.Abzw"),
        (DeDkst => "de.Dkst"),
        (DeUest => "de.Üst"),
        (DeUehst => "de.Uehst"),
        (DeAwanst => "de.Awanst"),
        (DeAnst => "de.Anst"),
        (DeLdst => "de.Ldst"),
        (DeAhst => "de.Ahst"),
        (DeGnst => "de.Gnst"),
        (DeGa => "de.Ga"),
        (DeStw => "de.Stw"),
        (DePo => "de.Po"),
        (DeGlgr => "de.Glgr"),
        (DeEGr => "de.EGr"),
        (DeLGr => "de.LGr"),
        (DeStrw => "de.Strw"),
        (DeTp => "de.Tp"),
        (DeGp => "de.Gp"),
        (DeUst => "de.Ust"),
        (DeMuseum => "de.Museum"),

        (DkSt => "dk.St"),
        (DkT => "dk.T"),
        (DkSmd => "dk.Smd"),
        (DkGr => "dk.Gr"),

        (NoS => "no.s"),
        (NoSp => "no.sp"),
        (NoHp => "no.hp"),
    }
}


//------------ Location ------------------------------------------------------

pub struct Location {
    line: LineRef,
    location: String
}

impl Location {
    pub fn line(&self) -> DocumentGuard<Line> {
        self.line.get()
    }

    pub fn location(&self) -> &str {
        &self.location
    }
}

impl FromYaml for Location {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let line = item.parse_mandatory("line", builder);
        let location = item.parse_mandatory("location", builder);
        item.exhausted(builder)?;

        Ok(Location {
            line: line?,
            location: location?
        })
    }
}


//------------ Service -------------------------------------------------------

mandatory_enum! {
    pub enum Service {
        (Full => "full"),
        (None => "none"),
        (Passenger => "passenger"),
        (Freight => "freight"),
    }
}


//------------ Side ----------------------------------------------------------

mandatory_enum! {
    pub enum Side {
        (Left => "left"),
        (Right => "right"),
        (Up => "up"),
        (Down => "down"),
        (Center => "center"),
    }
}

/*
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
*/


//------------ Site ----------------------------------------------------------

pub struct Site {
    path: PathRef,
    node: String,
    side: Option<Side>,
    distance: f64
}

impl Site {
    pub fn path(&self) -> DocumentGuard<Path> {
        self.path.get()
    }

    pub fn node(&self) -> &str {
        &self.node
    }

    pub fn side(&self) -> Option<Side> {
        self.side
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }
}

impl FromYaml for Site {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let path = item.parse_mandatory("path", builder);
        let node = item.parse_mandatory("node", builder);
        let side = item.parse_opt("side", builder);
        let distance = item.parse_mandatory("distance", builder);
        item.exhausted(builder)?;

        Ok(Site{path: path?, node: node?, side: side?, distance: distance?})
    }
}


//------------ Staff ---------------------------------------------------------

mandatory_enum! {
    pub enum Staff {
        (Full => "full"),
        (Agent => "agent"),
        (None => "none"),
    }
}


//------------ Status --------------------------------------------------------

mandatory_enum! {
    pub enum Status {
        (Open => "open"),
        (Suspended => "suspended"),
        (Closed => "closed"),
    }
}


//------------ DeRangklasse --------------------------------------------------

mandatory_enum! {
    pub enum DeRangklasse {
        (I => "I"),
        (Ii => "II"),
        (Iii => "III"),
        (Iv => "IV"),
        (V => "V"),
        (Vi => "VI"),
        (U => "U"),
        (S => "S"),
    }
}


//------------ PointRef ------------------------------------------------------

pub struct PointRef(DocumentRef);

impl PointRef {
    pub fn new(builder: &CollectionBuilder, key: &str, pos: Source) -> Self {
        PointRef(builder.ref_doc(key, pos, Some(DocumentType::Point)))
    }

    pub fn get(&self) -> DocumentGuard<Point> {
        self.0.get()
    }
}

impl FromYaml for PointRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(PointRef(builder.ref_doc(item.value(), item.source(),
                                    Some(DocumentType::Point))))
    }
}

impl FromYaml for Option<PointRef> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let pos = item.source();
        let value = item.parse::<Option<String>>(builder)?;
        Ok(value.map(|value| PointRef::new(builder, &value, pos)))
    }
}
