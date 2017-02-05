use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, Progress, ShortVec, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};
use super::organization::OrganizationRef;
use super::path::{Path, PathRef};
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
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let progress = item.parse_default("progress", builder);
        let label = item.parse_opt("label", builder);
        let events = item.parse_mandatory("events", builder);
        let points = item.parse_mandatory("points", builder);
        try_key!(item.exhausted(builder), key);
        Ok(Document::Line(Line {
            progress: try_key!(progress, key),
            label: try_key!(label, key),
            events: try_key!(events, key),
            points: try_key!(points, key),
            key: key,
        }))
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
    sections: ShortVec<Section>,
    document: Option<Sources>,
    sources: Sources,

    category: Option<ShortVec<Category>>,
    concession: Option<Concession>,
    contract: Option<Contract>,
    course: Option<ShortVec<CourseSegment>>,
    electrified: Option<Electrified>,
    freight: Option<Freight>,
    gauge: Option<ShortVec<u16>>,
    local_name: Option<LocalizedString>,
    name: Option<String>,
    note: Option<LocalizedString>,
    operator: Option<ShortVec<OrganizationRef>>,
    owner: Option<ShortVec<OrganizationRef>>,
    passenger: Option<Passenger>,
    rails: Option<u8>,
    region: Option<ShortVec<OrganizationRef>>,
    reused: Option<ShortVec<LineRef>>,
    status: Option<Status>,
    tracks: Option<u8>,
    treaty: Option<Contract>,

    de_vzg: Option<String>
}

impl Event {
    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn sections(&self) -> &ShortVec<Section> {
        &self.sections
    }

    pub fn document(&self) -> Option<&Sources> {
        self.document.as_ref()
    }

    pub fn sources(&self) -> &Sources {
        &self.sources
    }

    pub fn category(&self) -> Option<&ShortVec<Category>> {
        self.category.as_ref()
    }

    pub fn concession(&self) -> Option<&Concession> {
        self.concession.as_ref()
    }

    pub fn contract(&self) -> Option<&Contract> {
        self.contract.as_ref()
    }

    pub fn course(&self) -> Option<&ShortVec<CourseSegment>> {
        self.course.as_ref()
    }

    pub fn electrified(&self) -> Option<&Electrified> {
        self.electrified.as_ref()
    }

    pub fn freight(&self) -> Option<Freight> {
        self.freight
    }

    pub fn gauge(&self) -> Option<&ShortVec<u16>> {
        self.gauge.as_ref()
    }

    pub fn local_name(&self) -> Option<&LocalizedString> {
        self.local_name.as_ref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn operator(&self) -> Option<&ShortVec<OrganizationRef>> {
        self.operator.as_ref()
    }

    pub fn owner(&self) -> Option<&ShortVec<OrganizationRef>> {
        self.owner.as_ref()
    }

    pub fn passenger(&self) -> Option<Passenger> {
        self.passenger
    }

    pub fn rails(&self) -> Option<u8> {
        self.rails
    }

    pub fn region(&self) -> Option<&ShortVec<OrganizationRef>> {
        self.region.as_ref()
    }

    pub fn reused(&self) -> Option<&ShortVec<LineRef>> {
        self.reused.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    pub fn tracks(&self) -> Option<u8> {
        self.tracks
    }

    pub fn treaty(&self) -> Option<&Contract> {
        self.treaty.as_ref()
    }

    pub fn de_vzg(&self) -> Option<&str> {
        self.de_vzg.as_ref().map(AsRef::as_ref)
    }
}

impl Event {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let date = item.parse_opt("date", builder);
        let sections = Section::from_event_yaml(&mut item, builder);
        let document = item.parse_opt("document", builder);
        let sources = Sources::from_opt_yaml(item.optional_key("sources"),
                                             builder);
        let category = item.parse_opt("category", builder);
        let concession = Concession::from_yaml(item.optional_key("concession"),
                                               builder);
        let contract = Contract::from_yaml(item.optional_key("contract"),
                                           builder);
        let course = item.parse_opt("course", builder);
        let electrified = item.parse_opt("electrified", builder);
        let freight = item.parse_opt("freight", builder);
        let gauge = item.parse_opt("gauge", builder);
        let local_name = item.parse_opt("local_name", builder);
        let name = item.parse_opt("name", builder);
        let note = item.parse_opt("note", builder);
        let operator = item.parse_opt("operator", builder);
        let owner = item.parse_opt("owner", builder);
        let passenger = item.parse_opt("passenger", builder);
        let rails = item.parse_opt("rails", builder);
        let region = item.parse_opt("region", builder);
        let reused = item.parse_opt("reused", builder);
        let service = item.parse_opt("service", builder);
        let status = item.parse_opt("status", builder);
        let tracks = item.parse_opt("tracks", builder);
        let treaty = Contract::from_yaml(item.optional_key("treaty"),
                                         builder);
        let de_vzg = item.parse_opt("de.VzG", builder);
        item.exhausted(builder)?;

        // Obsolete document keys in concession, contract, and treaty.
        let mut document = document?;
        let (concession, concession_doc) = concession?;
        let (contract, contract_doc) = contract?;
        let (treaty, treaty_doc) = treaty?;
        if concession_doc.is_some() {
            if document.is_some() {
                builder.str_error(item.source(),
                                  "collision of obsolute document keys");
                return Err(())
            }
            document = concession_doc;
        }
        if contract_doc.is_some() {
            if document.is_some() {
                builder.str_error(item.source(),
                                  "collision of obsolute document keys");
                return Err(())
            }
            document = contract_doc;
        }
        if treaty_doc.is_some() {
            if document.is_some() {
                builder.str_error(item.source(),
                                  "collision of obsolute document keys");
                return Err(())
            }
            document = treaty_doc;
        }

        // Obsolete "service".
        let mut freight = freight?;
        let mut passenger = passenger?;
        if let Some(service) = service? {
            let (f, p) = match service {
                Service::None => (Freight::None, Passenger::None),
                Service::Freight => (Freight::Full, Passenger::None),
                Service::Passenger => (Freight::None, Passenger::Full),
                Service::Full => (Freight::Full, Passenger::Full),
            };
            freight = freight.or(Some(f));
            passenger = passenger.or(Some(p));
        }

        Ok(Event {
            date: date?,
            sections: sections?,
            document: document,
            sources: sources?,
            category: category?,
            concession: concession,
            contract: contract,
            course: course?,
            electrified: electrified?,
            freight: freight,
            gauge: gauge?,
            local_name: local_name?,
            name: name?,
            note: note?,
            operator: operator?,
            owner: owner?,
            passenger: passenger,
            rails: rails?,
            region: region?,
            reused: reused?,
            status: status?,
            tracks: tracks?,
            treaty: treaty,
            de_vzg: de_vzg?,
        })
    }
}


//------------ Section -------------------------------------------------------

pub struct Section(Option<PointRef>, Option<PointRef>);

impl Section {
    pub fn start(&self) -> Option<DocumentGuard<Point>> {
        self.0.as_ref().map(PointRef::get)
    }

    pub fn end(&self) -> Option<DocumentGuard<Point>> {
        self.1.as_ref().map(PointRef::get)
    }

    pub fn from_event_yaml(event: &mut Item<Mapping>,
                           builder: &CollectionBuilder)
                           -> Result<ShortVec<Self>, ()> {
        // We want to look at all three keys before erroring out. Hence the
        // slightly odd way.
        let start = event.parse_opt::<Item<String>>("start", builder);
        let end = event.parse_opt::<Item<String>>("end", builder);
        let sections = event.optional_key("sections");
        let start = start?;
        let end = end?;

        if let Some(sections) = sections {
            if start.is_some() || end.is_some() {
                builder.error((event.source(),
                               String::from("'start' and 'end' are not \
                                             allowed with 'section'")));
                return Err(())
            }
            sections.parse(builder)
        }
        else {
            let start = start.map(|start| {
                PointRef::new(builder, start.value(), start.source())
            });
            let end = end.map(|end| {
                PointRef::new(builder, end.value(), end.source())
            });
            Ok(ShortVec::One(Section(start, end)))
        }
    }
}

impl FromYaml for Section {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let start = item.parse_opt("start", builder);
        let end = item.parse_opt("end", builder);
        Ok(Section(start?, end?))
    }
}


//------------ Category ------------------------------------------------------

mandatory_enum! {
    pub enum Category {
        (DeHauptbahn => "de.Hauptbahn"),
        (DeNebenbahn => "de.Nebenbahn"),
        (DeKleinbahn => "de.Kleinbahn"),
        (DeAnschl => "de.Anschl"),
        (DeBfgleis => "de.Bfgleis"),
        (DeStrab => "de.Strab"),
    }
}


//------------ Concession ---------------------------------------------------

pub struct Concession {
    by: ShortVec<OrganizationRef>,
    to: ShortVec<OrganizationRef>,
    until: Option<Date>,
}

impl Concession {
    pub fn by(&self) -> &ShortVec<OrganizationRef> {
        &self.by
    }

    pub fn to(&self) -> &ShortVec<OrganizationRef> {
        &self.to
    }

    pub fn until(&self) -> Option<&Date> {
        self.until.as_ref()
    }
}

impl Concession {
    fn from_yaml(item: Option<ValueItem>, builder: &CollectionBuilder)
                 -> Result<(Option<Self>, Option<Sources>), ()> {
        let item = if let Some(item) = item { item }
                   else { return Ok((None, None)) };
        let mut item = item.into_mapping(builder)?;
        let by = ShortVec::from_opt_yaml(item.optional_key("by"), builder);
        let to = ShortVec::from_opt_yaml(item.optional_key("for"), builder);
        let until = item.parse_opt("until", builder);
        let document = item.parse_opt("document", builder);

        Ok((Some(Concession{by: by?, to: to?, until: until?}),
            document?))
    }
}


pub struct Contract {
    parties: ShortVec<OrganizationRef>,
}

impl Contract {
    pub fn parties(&self) -> &ShortVec<OrganizationRef> {
        &self.parties
    }
}

impl Contract {
    fn from_yaml(item: Option<ValueItem>, builder: &CollectionBuilder)
                 -> Result<(Option<Self>, Option<Sources>), ()> {
        let item = if let Some(item) = item { item }
                   else { return Ok((None, None)) };
        let mut item = item.into_mapping(builder)?;
        let parties = item.parse_mandatory("parties", builder);
        let document = item.parse_opt("document", builder);

        Ok((Some(Contract{parties: parties?}), document?))
    }
}


//------------ CourseSegment -------------------------------------------------

pub struct CourseSegment {
    path: PathRef,
    start: String,
    end: String,
    offset: f64,
}

impl CourseSegment {
    pub fn path(&self) -> DocumentGuard<Path> {
        self.path.get()
    }

    pub fn start(&self) -> &str {
        &self.start
    }

    pub fn end(&self) -> &str {
        &self.end
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }
}

impl FromYaml for CourseSegment {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let path = item.parse_mandatory("path", builder);
        let start = item.parse_mandatory("start", builder);
        let end = item.parse_mandatory("end", builder);
        let offset = match item.parse_opt("offset", builder)? {
            Some(offset) => offset,
            None => 0.
        };

        Ok(CourseSegment {
            path: path?,
            start: start?,
            end: end?,
            offset: offset
        })
    }
}


//------------ Voltage -------------------------------------------------------

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Voltage {
    Ac {
        volts: u32,
        frequency: f64,
        phases: u8,
    },
    Dc {
        volts: u32,
    },
}

impl Voltage {
    fn ac_from_yaml(item: &mut Item<Mapping>, builder: &CollectionBuilder)
                    -> Result<Self, ()> {
        let volts = item.parse_mandatory("voltage", builder);
        let frequency = item.parse_mandatory("frequency", builder);
        let phases = item.parse_opt("phases", builder)?
                         .unwrap_or(2);

        Ok(Voltage::Ac{volts: volts?, frequency: frequency?, phases: phases})
    }

    fn dc_from_yaml(item: &mut Item<Mapping>, builder: &CollectionBuilder)
                    -> Result<Self, ()> {
        let volts = item.parse_mandatory("voltage", builder);

        Ok(Voltage::Dc{volts: volts?})
    }

    fn from_yaml_mapping(item: &mut Item<Mapping>,
                         builder: &CollectionBuilder)
                         -> Result<Option<Self>, ()> {
        let vtype = item.parse_mandatory::<Item<String>>("type", builder)?;
        let (value, source) = vtype.into_inner();
        match value.as_ref() {
            "none" => Ok(None),
            "AC" => Ok(Some(Self::ac_from_yaml(item, builder)?)),
            "DC" => Ok(Some(Self::dc_from_yaml(item, builder)?)),
            _ => {
                builder.error((source,
                               format!("invalid voltage type '{}'", value)));
                Err(())
            }
        }
    }
}

impl FromYaml for Option<Voltage> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let res = Voltage::from_yaml_mapping(&mut item, builder)?;
        item.exhausted(builder)?;
        Ok(res)
    }
}


//------------ ContactRail ---------------------------------------------------

mandatory_enum! {
    pub enum ContactRail {
        (Top => "top"),
        (Inside => "inside"),
        (Outside => "outside"),
        (Bottom => "bottom"),
    }
}


//------------ ElectricRail --------------------------------------------------

pub struct ElectricRail {
    voltage: Voltage,
    rails: u8,
    contact: ContactRail
}

impl ElectricRail {
    pub fn voltage(&self) -> &Voltage {
        &self.voltage
    }

    pub fn rails(&self) -> u8 {
        self.rails
    }

    pub fn contact(&self) -> ContactRail {
        self.contact
    }
}

impl FromYaml for Option<ElectricRail> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        match Voltage::from_yaml_mapping(&mut item, builder)? {
            None => Ok(None),
            Some(voltage) => {
                let rails = item.parse_opt("rails", builder)
                                .map(|r| r.unwrap_or(1));
                let contact = item.parse_mandatory("contact", builder);

                Ok(Some(ElectricRail {
                    voltage: voltage,
                    rails: rails?,
                    contact: contact?
                }))
            }
        }
    }
}


//------------ Electrified ---------------------------------------------------

pub struct Electrified {
    rail: Option<Option<ElectricRail>>,
    overhead: Option<Option<Voltage>>,
}

impl Electrified {
    pub fn rail(&self) -> Option<Option<&ElectricRail>> {
        match self.rail {
            Some(Some(ref v)) => Some(Some(v)),
            Some(None) => Some(None),
            None => None
        }
    }

    pub fn overhead(&self) -> Option<Option<&Voltage>> {
        match self.overhead {
            Some(Some(ref v)) => Some(Some(v)),
            Some(None) => Some(None),
            None => None
        }
    }
}

impl FromYaml for Electrified {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let rail = item.parse_opt("rail", builder);
        let overhead = item.parse_opt("overhead", builder);
        item.exhausted(builder)?;

        Ok(Electrified {
            rail: rail?,
            overhead: overhead?,
        })
    }
}


//------------ Freight -------------------------------------------------------

mandatory_enum! {
    pub enum Freight {
        (None => "none"),
        (Restricted => "restricted"),
        (Full => "full"),
    }
}


//------------ Passenger -----------------------------------------------------

mandatory_enum! {
    pub enum Passenger {
        (None => "none"),
        (Restricted => "restricted"),
        (Historic => "historic"),
        (Seasonal => "seasonal"),
        (Tourist => "tourist"),
        (Full => "full"),
    }
}

//------------ Service -------------------------------------------------------

mandatory_enum! {
    pub enum Service {
        (None => "none"),
        (Passenger => "passenger"),
        (Freight => "freight"),
        (Full => "full"),
    }
}


//------------ Status --------------------------------------------------------

mandatory_enum! {
    pub enum Status {
        (Planned => "planned"),
        (Construction => "construction"),
        (Open => "open"),
        (Suspended => "suspended"),
        (Reopened => "reopened"),
        (Closed => "closed"),
        (Removed => "removed"),
        (Released => "released"),
    }
}


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
                                   Some(DocumentType::Line))))
    }
}

