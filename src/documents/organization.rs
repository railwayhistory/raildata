use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, Progress, ShortVec, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};


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
                     -> Result<Document, Option<String>> {
        let subtype = item.parse_default("subtype", builder);
        let progress = item.parse_default("progress", builder);
        let events = item.parse_mandatory("events", builder);
        try_key!(item.exhausted(builder), key);
        Ok(Document::Organization(Organization {
            subtype: try_key!(subtype, key),
            progress: try_key!(progress, key),
            events: try_key!(events, key),
            key: key,
        }))
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

optional_enum! {
    pub enum Subtype {
        (Company => "company"),
        (Country => "country"),
        (Department => "department"),
        (Person => "person"),
        (Region => "region"),
        (Misc => "misc"),

        default Misc
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
        let sources = Sources::from_opt_yaml(item.optional_key("sources"),
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
                                           Some(DocumentType::Organization))))
    }
}

