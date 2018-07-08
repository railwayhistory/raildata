
use ::load::yaml::{FromYaml, Mapping, Value};
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::types::{EventDate, Key, LanguageText, LocalText, List, Marked};
use super::{OrganizationLink, SourceLink};
use super::common::{Basis, Common, Progress};
use super::store::{LoadStore, Stored};


//------------ Organization --------------------------------------------------

#[derive(Clone, Debug)]
pub struct Organization {
    common: Common,
    subtype: Marked<Subtype>,
    events: EventList,
}

impl Organization {
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

    pub fn subtype(&self) -> Subtype {
        self.subtype.into_value()
    }
}

impl<'a> Stored<'a, Organization> {
    pub fn events(&self) -> Stored<'a, EventList> {
        self.map(|item| &item.events)
    }
}

impl Organization {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take("subtype", context, report);
        let events = doc.take("events", context, report);
        doc.exhausted(report)?;
        Ok(Organization {
            common: common?,
            subtype: subtype?,
            events: events?,
        })
    }

    pub fn verify(&self, _report: &mut StageReporter) {
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Company: "company" }
        { Country: "country" }
        { Person: "person" }
        { Place: "place" }
        { Region: "region" }
    }
}


//------------ EventList -----------------------------------------------------

pub type EventList = List<Event>;


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Event {
    // Meta attributes
    date: EventDate,
    document: List<Marked<SourceLink>>,
    source: List<Marked<SourceLink>>,
    basis: List<Basis>,
    note: Option<LanguageText>,

    // Organization property attributes
    domicile: List<Marked<OrganizationLink>>,
    master: Option<Marked<OrganizationLink>>,
    name: Option<LocalText>,
    owner: Option<List<Marked<OrganizationLink>>>,
    property: Option<Property>,
    short_name: Option<LocalText>,
    status: Option<Status>,
    successor: Option<Marked<OrganizationLink>>,
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

    pub fn basis(&self) -> Stored<'a, List<Basis>> {
        self.map(|item| &item.basis)
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.access().note.as_ref()
    }

    pub fn domicile(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.domicile)
    }

    pub fn master(&self) -> Option<&Organization> {
        self.map_opt(|item| item.master.as_ref()).map(|x| x.follow())
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.access().name.as_ref()
    }

    pub fn owner(
        &self
    ) -> Option<Stored<'a, List<Marked<OrganizationLink>>>> {
        self.map_opt(|item| item.owner.as_ref())
    }

    pub fn property(&self) -> Option<Stored<'a, Property>> {
        self.map_opt(|item| item.property.as_ref())
    }

    pub fn short_name(&self) -> Option<&LocalText> {
        self.access().short_name.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.access().status
    }

    pub fn successor(&self) -> Option<&Organization> {
        self.map_opt(|item| item.successor.as_ref()).map(|x| x.follow())
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
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);
        let domicile = value.take_default("domicile", context, report);
        let master = value.take_opt("master", context, report);
        let name = value.take_opt("name", context, report);
        let owner = value.take_default("owner", context, report);
        let property = value.take_opt("property", context, report);
        let short_name = value.take_opt("short_name", context, report);
        let status = value.take_opt("status", context, report);
        let successor = value.take_opt("successor", context, report);
        value.exhausted(report)?;
        Ok(Event {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,
            domicile: domicile?,
            master: master?,
            name: name?,
            owner: owner?,
            property: property?,
            short_name: short_name?,
            status: status?,
            successor: successor?,
        })
    }
}


//------------ Property ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Property {
    role: Marked<PropertyRole>,
    constructor: List<Marked<OrganizationLink>>,
    operator: List<Marked<OrganizationLink>>,
    owner: List<Marked<OrganizationLink>>,
}

impl<'a> Stored<'a, Property> {
    pub fn role(&self) -> PropertyRole {
        self.access().role.into_value()
    }

    pub fn constructor(
        &self
    ) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.constructor)
    }

    pub fn operator(
        &self
    ) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.operator)
    }

    pub fn owner(
        &self
    ) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.owner)
    }
}

impl FromYaml<LoadStore> for Property {
    fn from_yaml(
        value: Value,
        context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let role = value.take("role", context, report);
        let constructor = value.take_default("constructor", context, report);
        let owner = value.take_default("owner", context, report);
        let operator = value.take_default("operator", context, report);
        value.exhausted(report)?;
        Ok(Property {
            role: role?,
            constructor: constructor?,
            owner: owner?,
            operator: operator?,
        })
    }
}


//------------ PropertyRole --------------------------------------------------

data_enum! {
    pub enum PropertyRole {
        { Constructur: "constructor" }
        { Owner: "owner" }
        { Operator: "operator" }
    }
}


//------------ Status --------------------------------------------------------

data_enum! {
    pub enum Status {
        { Forming: "forming" }
        { Open: "open" }
        { Closed: "closed" }
    }
}

