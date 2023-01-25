
use serde::{Deserialize, Serialize};
use crate::store::XrefsStore;
use crate::document::entity;
use crate::document::combined::EntityLink;
use crate::load::report::{Failed, PathReporter};
use crate::types::{EventDate, List};
use super::data::{Data, Section, SectionList};


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    property: PropertyList,
}

impl Meta {
    pub fn generate(
        data: &Data, store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        Ok(Meta {
            property: PropertyList::generate(data, store),
        })
    }
}


//------------ PropertyList --------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct PropertyList {
    items: (EventDate, List<PropertySection>)
}

impl PropertyList {
    fn generate(data: &Data, _store: &XrefsStore) -> Self {
        let res = Self::default();
        let (_date, _events) = match property::find_first(data) {
            Some(some) => some,
            None => return res,
        };

        

        unimplemented!()
    }
}


//------------ PropertySection -----------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PropertySection {
    section: Section,
    operators: Option<List<EntityLink>>,
    owners: Option<List<EntityLink>>,
}


//============ Generating the Property List ==================================

#[allow(dead_code, unused_variables)] // XXX
mod property {
    use super::*;

    pub enum Event {
        Explicit(ExplicitEvent),
        Entity(EntityEvent),
    }

    pub struct ExplicitEvent {
        sections: SectionList,
        operator: Option<List<EntityLink>>,
        owner: Option<List<EntityLink>>,
    }

    pub struct EntityEvent {
        entity: entity::Link,
        property: entity::Property,
    }

    pub fn find_first(
        data: &Data
    ) -> Option<(EventDate, Vec<ExplicitEvent>)> {
        unimplemented!()
    }

    pub fn find_next(
        date: EventDate, data: &Data, store: &XrefsStore
    ) -> Option<(EventDate, Vec<Event>)> {
        unimplemented!()
    }
}

