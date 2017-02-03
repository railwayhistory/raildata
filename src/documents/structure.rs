use std::ops;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, Progress, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};


//------------ Structure -----------------------------------------------------

pub struct Structure {
    key: String,
    subtype: Subtype,
    progress: Progress,
    events: Events,
}

impl Structure {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let subtype = item.parse_mandatory("subtype", builder);
        let progress = item.parse_default("progress", builder);
        let events = item.parse_mandatory("events", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Document::Structure(Structure {
            subtype: try_key!(subtype, key),
            progress: try_key!(progress, key),
            events: try_key!(events, key),
            key: key,
        }))
    }
}

impl Structure {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn events(&self) -> &Events {
        &self.events
    }
}


//------------ Subtype -------------------------------------------------------

mandatory_enum! {
    pub enum Subtype {
        (Tunnel => "tunnel"),
        (Bridge => "bridge"),
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
    length: Option<u32>,
    name: Option<String>,
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

    pub fn length(&self) -> Option<u32> {
        self.length
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }
}

impl FromYaml for Event {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let date = item.parse_opt("date", builder);
        let sources = item.parse_default("sources", builder);
        let local_name = item.parse_opt("local_name", builder);
        let length = item.parse_opt("length", builder);
        let name = item.parse_opt("name", builder);
        item.exhausted(builder)?;

        Ok(Event {
            date: date?,
            sources: sources?,
            local_name: local_name?,
            length: length?,
            name: name?,
        })
    }
}


//------------ StructureRef --------------------------------------------------

pub struct StructureRef(DocumentRef);

impl StructureRef {
    pub fn get(&self) -> DocumentGuard<Structure> {
        self.0.get()
    }
}

impl FromYaml for StructureRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(StructureRef(builder.ref_doc(item.value(), item.source(),
                                        Some(DocumentType::Structure))))
    }
}

