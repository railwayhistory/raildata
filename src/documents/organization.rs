use std::ops;
use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::{MarkedMapping, Value};
use super::common::{Basis, Common};
use super::links::{OrganizationLink, SourceLink};
use super::types::{EventDate, List, LanguageText, LocalText};


//------------ Organization --------------------------------------------------

#[derive(Clone, Debug)]
pub struct Organization {
    common: Common,
    subtype: Subtype,
    events: List<Event>,
}

impl Organization {
    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn events(&self) -> &List<Event> {
        &self.events
    }
}

impl Organization {
    pub fn construct<C>(common: Common, mut doc: MarkedMapping,
                        context: &mut C) -> Result<Self, Failed>
                     where C: Context {
        let subtype = doc.take("subtype", context);
        let events = doc.take("events", context);
        doc.exhausted(context)?;
        Ok(Organization { common,
            subtype: subtype?,
            events: events?,
        })
    }
}


impl ops::Deref for Organization {
    type Target = Common;

    fn deref(&self) -> &Common {
        &self.common
    }
}

impl ops::DerefMut for Organization {
    fn deref_mut(&mut self) -> &mut Common {
        &mut self.common
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


//------------ Event ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Event {
    // Meta attributes
    date: EventDate,
    document: List<SourceLink>,
    source: List<SourceLink>,
    basis: List<Basis>,
    note: Option<LanguageText>,

    // Organization property attributes
    domicile: List<OrganizationLink>,
    master: Option<OrganizationLink>,
    name: Option<LocalText>,
    owner: List<OrganizationLink>,
    property: Option<Property>,
    short_name: Option<LocalText>,
    status: Option<Status>,
    successor: Option<OrganizationLink>,
}

/// # Event Metadata Attributes
///
impl Event {
    pub fn date(&self) -> &EventDate {
        &self.date
    }

    pub fn document(&self) -> &List<SourceLink> {
        &self.document
    }

    pub fn source(&self) -> &List<SourceLink> {
        &self.source
    }

    pub fn basis(&self) -> &List<Basis> {
        &self.basis
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.note.as_ref()
    }
}

/// # Organization Property Attributes
///
impl Event {
    pub fn domicile(&self) -> &List<OrganizationLink> {
        &self.domicile
    }

    pub fn master(&self) -> Option<&OrganizationLink> {
        self.master.as_ref()
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.name.as_ref()
    }

    pub fn owner(&self) -> &List<OrganizationLink> {
        &self.owner
    }

    pub fn property(&self) -> Option<&Property> {
        self.property.as_ref()
    }

    pub fn short_name(&self) -> Option<&LocalText> {
        self.short_name.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    pub fn successor(&self) -> Option<&OrganizationLink> {
        self.successor.as_ref()
    }
}


impl Constructable for Event {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let date = value.take("date", context);
        let document = value.take_default("document", context);
        let source = value.take_default("source", context);
        let basis = value.take_default("basis", context);
        let note = value.take_opt("note", context);
        let domicile = value.take_default("domicile", context);
        let master = value.take_opt("master", context);
        let name = value.take_opt("name", context);
        let owner = value.take_default("owner", context);
        let property = value.take_opt("property", context);
        let short_name = value.take_opt("short_name", context);
        let status = value.take_opt("status", context);
        let successor = value.take_opt("successor", context);
        value.exhausted(context)?;
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
    role: PropertyRole,
    constructor: List<OrganizationLink>,
    owner: List<OrganizationLink>,
    operator: List<OrganizationLink>
}

impl Property {
    pub fn role(&self) -> PropertyRole {
        self.role
    }

    pub fn constructor(&self) -> &List<OrganizationLink> {
        &self.constructor
    }

    pub fn owner(&self) -> &List<OrganizationLink> {
        &self.owner
    }

    pub fn operator(&self) -> &List<OrganizationLink> {
        &self.operator
    }
}

impl Constructable for Property {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let role = value.take("role", context);
        let constructor = value.take_default("constructor", context);
        let owner = value.take_default("owner", context);
        let operator = value.take_default("operator", context);
        value.exhausted(context)?;
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
