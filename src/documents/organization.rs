use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, Sequence, ValueItem};
use super::common::{LocalizedString, Progress, ShortVec, Sources};
use super::date::Date;
use super::document::DocumentType;


//------------ Organization --------------------------------------------------

pub struct Organization {
    key: String,
    subtype: Subtype,
    progress: Progress,
    events: Events,
}

impl Organization {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Organization, ()> {
        let subtype = Subtype::from_yaml(item.optional_key("subtype"),
                                         builder);
        let progress = Progress::from_yaml(item.optional_key("progress"),
                                           builder);
        let events = Events::from_yaml(item.mandatory_key("events", builder)?
                                           .into_sequence(builder)?,
                                       builder);
        Ok(Organization {
            key: key,
            subtype: subtype?,
            progress: progress?,
            events: events?,
        })
    }
}

impl Organization {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn events(&self) -> &[Event] {
        &self.events.0
    }
}


//------------ Subtype -------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Subtype {
    Company,
    Country,
    Department,
    Person,
    Region,
    Misc,
}

impl Subtype {
    fn from_yaml(item: Option<ValueItem>, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        if let Some(item) = item {
            let item = item.into_string_item(builder)?;
            match item.as_ref().as_ref() {
                "company" => Ok(Subtype::Company),
                "country" => Ok(Subtype::Country),
                "department" => Ok(Subtype::Department),
                "person" => Ok(Subtype::Person),
                "region" => Ok(Subtype::Region),
                "misc" => Ok(Subtype::Misc),
                _ => {
                    builder.error((item.source(),
                                   format!("invalid subtype value '{}'",
                                           item.value())));
                    Err(())
                }
            }
        }
        else {
            Ok(Subtype::Misc)
        }
    }
}

impl Default for Subtype {
    fn default() -> Self {
        Subtype::Misc
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
    sources: Sources,

    local_name: Option<LocalizedString>,
    local_short_name: Option<LocalizedString>,
    master: Option<OrganizationRef>,
    merged: Option<OrganizationRef>,
    name: Option<String>,
    note: Option<LocalizedString>,
    owner: Option<ShortVec<OrganizationRef>>,
    short_name: Option<String>,
    status: Option<Status>,
}


impl Event {
    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn sources(&self) -> &Sources {
        &self.sources
    }

    pub fn local_name(&self) -> Option<&LocalizedString> {
        self.local_name.as_ref()
    }

    pub fn local_short_name(&self) -> Option<&LocalizedString> {
        self.local_short_name.as_ref()
    }

    pub fn master(&self) -> Option<&OrganizationRef> {
        self.master.as_ref()
    }

    pub fn merged(&self) -> Option<&OrganizationRef> {
        self.merged.as_ref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(ops::Deref::deref)
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn owner(&self) -> Option<&ShortVec<OrganizationRef>> {
        self.owner.as_ref()
    }

    pub fn short_name(&self) -> Option<&str> {
        self.short_name.as_ref().map(ops::Deref::deref)
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }
}


impl Event {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let date = item.parse_opt("date", builder);
        let sources = Sources::from_yaml(item.optional_key("sources"),
                                         builder);
        let local_name = item.parse_opt("local_name", builder);
        let local_short_name = item.parse_opt("local_short_name", builder);
        let master = item.parse_opt("master", builder);
        let merged = item.parse_opt("merged", builder);
        let name = item.parse_opt("name", builder);
        let note = item.parse_opt("note", builder);
        let owner = item.parse_opt("owner", builder);
        let short_name = item.parse_opt("short_name", builder);
        let status = item.parse_opt("status", builder);
        Ok(Event {
            date: date?,
            sources: sources?,
            local_name: local_name?,
            local_short_name: local_short_name?,
            master: master?,
            merged: merged?,
            name: name?,
            note: note?,
            owner: owner?,
            short_name: short_name?,
            status: status?,
        })
    }
}


//------------ Status --------------------------------------------------------

mandatory_enum! {
    pub enum Status {
        (Open => "open"),
        (Closed => "closed"),
    }
}


//------------ OrganizationRef -----------------------------------------------

pub struct OrganizationRef(DocumentRef);

impl OrganizationRef {
    pub fn get(&self) -> DocumentGuard<Organization> {
        self.0.get()
    }
}

impl FromYaml for OrganizationRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(OrganizationRef(builder.ref_doc(item.value(), item.source(),
                                           DocumentType::Organization)))
    }
}

