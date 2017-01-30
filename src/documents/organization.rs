use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::error::{ErrorGatherer};
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
                     collection: &mut CollectionBuilder,
                     errors: &ErrorGatherer) -> Result<Organization, ()> {
        let subtype = Subtype::from_yaml(item.optional_key("subtype"),
                                         errors);
        let progress = Progress::from_yaml(item.optional_key("progress"),
                                           errors);
        let events = Events::from_yaml(item.mandatory_key("events", errors)?
                                           .into_sequence(errors)?,
                                       collection, errors);
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
    fn from_yaml(item: Option<ValueItem>, errors: &ErrorGatherer)
                 -> Result<Self, ()> {
        if let Some(item) = item {
            let item = item.into_string_item(errors)?;
            match item.as_ref().as_ref() {
                "company" => Ok(Subtype::Company),
                "country" => Ok(Subtype::Country),
                "department" => Ok(Subtype::Department),
                "person" => Ok(Subtype::Person),
                "region" => Ok(Subtype::Region),
                "misc" => Ok(Subtype::Misc),
                _ => {
                    errors.add((item.source(),
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
    fn from_yaml(item: Sequence, collection: &mut CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<Self, ()> {
        let mut res = Some(Vec::new());
        for event in item {
            if let Ok(event) = Event::from_yaml(event, collection, errors) {
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
    fn from_yaml(item: ValueItem, collection: &mut CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<Self, ()> {
        let mut item = item.into_mapping(errors)?;
        let date = item.parse_opt("date", collection, errors);
        let sources = Sources::from_yaml(item.optional_key("sources"),
                                         collection, errors);
        let local_name = item.parse_opt("local_name", collection, errors);
        let local_short_name = item.parse_opt("local_short_name", collection,
                                              errors);
        let master = item.parse_opt("master", collection, errors);
        let merged = item.parse_opt("merged", collection, errors);
        let name = item.parse_opt("name", collection, errors);
        let note = item.parse_opt("note", collection, errors);
        let owner = item.parse_opt("owner", collection, errors);
        let short_name = item.parse_opt("short_name", collection, errors);
        let status = item.parse_opt("status", collection, errors);
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
    fn from_yaml(item: ValueItem, collection: &mut CollectionBuilder,
                 errs: &ErrorGatherer) -> Result<Self, ()> {
        let item = item.into_string_item(errs)?;
        Ok(OrganizationRef(collection.ref_doc(item.value(), item.source(),
                                              DocumentType::Organization)))
    }
}

