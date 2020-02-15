
use std::sync::Arc;
use crate::library::{LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{EventDate, Key, LanguageText, List, LocalText, Marked, Set};
use super::{LineLink, PathLink, PointLink, SourceLink};
use super::common::{Common, Progress};


//------------ Point ---------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    // Attributes
    pub common: Common,
    pub events: List<Event>,
    pub records: List<Event>,
    pub junction: Option<Marked<bool>>,
    pub subtype: Marked<Subtype>,

    // Crosslinked and derived data.
    pub lines: List<LineLink>,
    pub connections: Set<PointLink>,
}

/// # Data Access
///
impl Point {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    /// Returns whether the point is a junction.
    ///
    /// A junction is a point that connects lines. Any point can be declared
    /// a junction or not a junction via the `junction` point attribute. If
    /// this attribute is missing, it becomes a junction if it is listed in
    /// the `points` attribute of more than one line or if it is connected to
    /// some other point via its or the other point’s `connection` attribute.
    pub fn is_junction(&self) -> bool {
        match self.junction {
            Some(value) => value.into_value(),
            None => {
                !self.connections.is_empty() || self.lines.len() > 1
            }
        } 
    }

    /// Returns whether the point can’t be a junction.
    ///
    /// This happens if it has the `junction` attribute set to false.
    pub fn is_never_junction(&self) -> bool {
        !self.junction.map(Marked::into_value).unwrap_or(true)
    }

    /// Returns the current name.
    ///
    /// This is just a temporary placeholder that doesn’t regard languages and
    /// jurisdictions. It just picks the first name from the newest available
    /// attribute.
    pub fn name(&self) -> &str {
        for event in self.events.iter().rev() {
            if let Some(ref name) = event.name {
                return name.first()
            }
            if let Some(ref name) = event.designation {
                return name.first()
            }
        }
        self.key().as_str()
    }
}


/// # Loading
///
impl Point {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &LibraryBuilder,
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
        let mut records: EventList = records?.unwrap_or_default();
        records.sort_by(|left, right| left.date.sort_cmp(&right.date));

        Ok(Point {
            common: common?,
            events,
            records,
            junction: junction?,
            subtype: subtype?,
            lines: List::new(),
            connections: Set::new(),
        })
    }

    //--- Crosslinking

    pub fn crosslink(
        &mut self,
        link: PointLink,
        library: &LibraryMut,
        report: &mut StageReporter
    ) {
        // event::connection
        //
        // In order to get a complete set of connections, we produce a set of
        // all connections and add that full set (sans self) to all points.
        let mut set = Set::new();
        for event in &self.events {
            if let Some(ref conns) = event.connection {
                for conn in conns {
                    if conn.into_value() == link {
                        report.error_at(
                            self.origin().at(conn.location()),
                            OwnConnection
                        );
                        continue;
                    }
                    set.insert(conn.into_value());
                }
            }
        }
        if !set.is_empty() {
            self.connections.merge(&set);
            set.insert(link);
            let set = Arc::new(set);
            for target in set.iter() {
                let set = set.clone();
                let target = target.clone();
                target.update(library, move |point| {
                    for link in set.iter() {
                        if *link != target {
                            point.connections.insert(*link);
                        }
                    }
                })
            }
        }
    }

    pub fn add_line(&mut self, line: LineLink) {
        self.lines.push(line);
    }

    /*
    pub fn verify(&self, _report: &mut StageReporter) {
    }
    */
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub date: EventDate,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
    pub note: Option<LanguageText>,

    pub category: Option<Set<Category>>,
    pub connection: Option<List<Marked<PointLink>>>,
    pub designation: Option<LocalText>,
    pub express: Option<ServiceRate>,
    pub goods: Option<ServiceRate>,
    pub location: Option<Location>,
    pub luggage: Option<ServiceRate>,
    pub master: Option<Option<List<Marked<PointLink>>>>,
    pub merged: Option<Marked<PointLink>>,
    pub name: Option<LocalText>,
    pub passenger: Option<ServiceRate>,
    pub plc: Option<Plc>,
    pub public_name: Option<List<LocalText>>,
    pub site: Option<Site>,
    pub short_name: Option<LocalText>,
    pub staff: Option<Staff>,
    pub status: Option<Status>,

    pub service: Option<Service>,
    pub split_from: Option<Marked<PointLink>>,

    pub de_ds100: Option<DeDs100>,
    pub de_dstnr: Option<DeDstnr>,
    pub de_lknr: Option<List<DeLknr>>,
    pub de_name16: Option<DeName16>,
    pub de_rang: Option<DeRang>,
    pub de_vbl: Option<DeVbl>,

    pub dk_ref: Option<Marked<String>>,

    pub no_fs: Option<Marked<String>>,
    pub no_njk: Option<Marked<String>>,
    pub no_nsb: Option<Marked<String>>,
}

impl FromYaml<LibraryBuilder> for Event {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
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
        let express = value.take_opt("express", context, report);
        let goods = value.take_opt("goods", context, report);
        let location = value.take_opt("location", context, report);
        let luggage = value.take_opt("luggage", context, report);
        let master = value.take_opt("master", context, report);
        let merged = value.take_opt("merged", context, report);
        let name = value.take_opt("name", context, report);
        let passenger = value.take_opt("passenger", context, report);
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
            express: express?,
            goods: goods?,
            location: location?,
            luggage: luggage?,
            master: master?,
            merged: merged?,
            name: name?,
            passenger: passenger?,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Location(pub List<(Marked<LineLink>, Option<Marked<String>>)>);

impl FromYaml<LibraryBuilder> for Location {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
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
            let key = LineLink:: build(key, context, report);
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
pub struct Site(pub List<(Marked<PathLink>, Marked<String>)>);

impl FromYaml<LibraryBuilder> for Site {
    fn from_yaml(
        value: Value,
        context: &LibraryBuilder,
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
            let key = PathLink::build(key, context, report);
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

pub type DeDstnr = Marked<Option<String>>;


//------------ DeLknr --------------------------------------------------------

pub type DeLknr = Marked<String>; 


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


//------------ DeVbl ---------------------------------------------------------

pub type DeVbl = Marked<String>;


//------------ DeName16 ------------------------------------------------------

pub type DeName16 = Marked<String>;


//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="point listed as its own connection")]
pub struct OwnConnection;

