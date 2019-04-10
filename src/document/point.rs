
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::store::{
    LoadStore, Stored, UpdateStore,
    LineLink, PathLink, PointLink, SourceLink
};
use ::types::{EventDate, Key, LanguageText, List, LocalText, Marked, Set};
use super::common::{Common, Progress};


//------------ Point ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Point {
    // Attributes
    common: Common,
    events: List<Event>,
    junction: Option<Marked<bool>>,
    subtype: Marked<Subtype>,

    // Crosslinked data.
    lines: List<LineLink>,
    connections: Set<PointLink>,
}


/// # Data Access
///
impl Point {
    pub fn common(&self) -> &Common {
        &self.common
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

    /// Returns whether the point is a junction.
    ///
    /// A junction is a point that connects lines. Any point can be declared
    /// a junction or not a junction via the `junction` point attribute. If
    /// this attribute is missing, it becomes a junction if it is listed in
    /// the `points` attribute of more than one line or if it is connected to
    /// some other point via its or the other point’s `connection` attribute.
    pub fn junction(&self) -> bool {
        match self.junction {
            Some(value) => value.into_value(),
            None => {
                !self.connections.is_empty() || self.lines.len() > 1
            }
        } 
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype.into_value()
    }
}

impl<'a> Stored<'a, Point> {
    pub fn events(&self) -> Stored<'a, EventList> {
        self.map(|item| &item.events)
    }
}


/// # Loading
///
impl Point {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let events = doc.take("events", context, report);
        let junction = doc.take_opt("junction", context, report);
        let subtype = doc.take_default("subtype", context, report);
        doc.exhausted(report)?;
        Ok(Point {
            common: common?,
            events: events?,
            junction: junction?,
            subtype: subtype?,
            lines: List::new(),
            connections: Set::new(),
        })
    }

    //--- Crosslinking

    pub fn crosslink(
        &mut self,
        _link: PointLink,
        _store: &mut UpdateStore,
        _report: &mut StageReporter
    ) {
        /*
        for event in &self.events {
            if let Some(ref conns) = event.connection {
                for conn in conns {
                    if conn.as_value().clone() == link {
                        // XXX Produce an error here!
                        continue
                    }
                    self.connections.insert(conn.as_value().clone());
                    conn.update(store, |point| {
                        point.connections.insert(link.clone());
                    })
                }
            }
        }
        */
    }

    pub fn add_line(&mut self, line: LineLink) {
        self.lines.push(line);
    }

    pub fn verify(&self, _report: &mut StageReporter) {
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


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Event {
    date: EventDate,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    note: Option<LanguageText>,

    category: Option<Set<Category>>,
    connection: Option<List<Marked<PointLink>>>,
    designation: Option<LocalText>,
    location: Option<Location>,
    master: Option<Option<List<Marked<PointLink>>>>,
    merged: Option<Marked<PointLink>>,
    name: Option<LocalText>,
    plc: Option<Plc>,
    public_name: Option<List<LocalText>>,
    site: Option<Site>,
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

impl<'a> Stored<'a, Event> {
    pub fn date(&self) -> &EventDate {
        &self.access().date
    }

    pub fn document(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.document)
    }

    pub fn source(&self) -> Stored<'a, List<Marked<SourceLink>>> {
        self.map(|item| &item.source)
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }

    pub fn category(&self) -> Option<&Set<Category>> {
        self.access().category.as_ref()
    }

    pub fn connection(&self) -> Option<Stored<'a, List<Marked<PointLink>>>> {
        self.map_opt(|item| item.connection.as_ref())
    }

    pub fn designation(&self) -> Option<&LocalText> {
        self.access().designation.as_ref()
    }

    pub fn location(&self) -> Option<Stored<'a, Location>> {
        self.map_opt(|item| item.location.as_ref())
    }

    pub fn master(
        &self
    ) -> Stored<'a, Option<Option<List<Marked<PointLink>>>>> {
        self.map(|item| &item.master)
    }

    pub fn merged(&self) -> Option<Stored<Point>> {
        self.map_opt(|item| item.merged.as_ref()).map(|x| x.follow())
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.access().name.as_ref()
    }

    pub fn plc(&self) -> Option<&Plc> {
        self.access().plc.as_ref()
    }

    pub fn public_name(&self) -> Option<&List<LocalText>> {
        self.access().public_name.as_ref()
    }

    pub fn site(&self) -> Option<Stored<'a, Site>> {
        self.map_opt(|item| item.site.as_ref())
    }

    pub fn short_name(&self) -> Option<&LocalText> {
        self.access().short_name.as_ref()
    }

    pub fn staff(&self) -> Option<Staff> {
        self.access().staff
    }

    pub fn status(&self) -> Option<Status> {
        self.access().status
    }

    pub fn service(&self) -> Option<Service> {
        self.access().service
    }

    pub fn split_from(&self) -> Option<Stored<Point>> {
        self.map_opt(|item| item.split_from.as_ref()).map(|x| x.follow())
    }

    pub fn de_ds100(&self) -> Option<&DeDs100> {
        self.access().de_ds100.as_ref()
    }

    pub fn de_dstnr(&self) -> Option<&DeDstnr> {
        self.access().de_dstnr.as_ref()
    }

    pub fn de_lknr(&self) -> Option<&List<DeLknr>> {
        self.access().de_lknr.as_ref()
    }

    pub fn de_name16(&self) -> Option<&DeName16> {
        self.access().de_name16.as_ref()
    }

    pub fn de_rang(&self) -> Option<&DeRang> {
        self.access().de_rang.as_ref()
    }

    pub fn de_vbl(&self) -> Option<&DeVbl> {
        self.access().de_vbl.as_ref()
    }

    pub fn dk_ref(&self) -> Option<&str> {
        self.access().dk_ref.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn no_fs(&self) -> Option<&str> {
        self.access().no_fs.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn no_njk(&self) -> Option<&str> {
        self.access().no_njk.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn no_nsb(&self) -> Option<&str> {
        self.access().no_nsb.as_ref().map(|x| x.as_value().as_ref())
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
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let note = value.take_opt("note", context, report);

        let category = value.take_opt("category", context, report);
        let connection = value.take_opt("connection", context, report);
        let designation = value.take_opt("designation", context, report);
        let location = value.take_opt("location", context, report);
        let master = value.take_opt("master", context, report);
        let merged = value.take_opt("merged", context, report);
        let name = value.take_opt("name", context, report);
        let plc = value.take_opt("PLC", context, report);
        let public_name = value.take_opt("public_name", context, report);
        let site = value.take_opt("site", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let staff = value.take_opt("staff", context, report);
        let status = value.take_opt("status", context, report);

        let service = value.take_opt("service", context, report);
        let split_from = value.take_opt("split_from", context, report);

        let de_ds100 = value.take_opt("de.DS100", context, report);
        let de_dstnr = value.take_opt("de.dstnr", context, report);
        let de_lknr = value.take_opt("de.lknr", context, report);
        let de_name16 = value.take_opt("de.name16", context, report);
        let de_rang = value.take_opt("de.rang", context, report);
        let de_vbl = value.take_opt("de.VBL", context, report);

        let dk_ref = value.take_opt("dk.ref", context, report);
        
        let no_fs = value.take_opt("no.fs", context, report);
        let no_njk = value.take_opt("no.NJK", context, report);
        let no_nsb = value.take_opt("no.NSB", context, report);

        value.exhausted(report)?;
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
        { DeRBGr: "de.RBGr" }

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

impl FromYaml<LoadStore> for Location {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut res = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(report)? {
            let key = match Marked::from_string(key, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue;
                }
            };
            let key = match LineLink::forge(key, context, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue
                }
            };
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
pub struct Site(List<(Marked<PathLink>, Marked<String>)>);

impl FromYaml<LoadStore> for Site {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut res = List::new();
        let mut err = false;
        for (key, value) in value.into_mapping(report)? {
            let key = match Marked::from_string(key, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue;
                }
            };
            let key = match PathLink::forge(key, context, report) {
                Ok(key) => key,
                Err(_) => {
                    err = true;
                    continue
                }
            };
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

