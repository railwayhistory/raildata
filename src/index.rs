
use std::fmt;
use std::collections::hash_map::{HashMap, Entry, Values};
use ::links::Permalink;
use ::load::path::Path;
use ::types::{Key, Location};


//------------ PrimaryIndex --------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct PrimaryIndex {
    documents: HashMap<Key, Permalink>,
}

impl PrimaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: Key, link: Permalink)
                  -> Result<(), DocumentExists> {
        match self.documents.entry(key) {
            Entry::Occupied(entry) => {
                Err(DocumentExists::from_link(entry.get()))
            }
            Entry::Vacant(entry) => {
                entry.insert(link);
                Ok(())
            }
        }
    }

    pub fn get(&self, key: &Key) -> Option<Permalink> {
        self.documents.get(key).map(Clone::clone)
    }

    pub fn values(&self) -> Values<Key, Permalink> {
        self.documents.values()
    }
}


//------------ KeyExists -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyExists;


//------------ DocumentExists ------------------------------------------------

#[derive(Clone, Debug)]
pub struct DocumentExists {
    key: Key,
    path: Path,
    location: Location
}

impl DocumentExists {
    pub fn from_link(link: &Permalink) -> Self {
        let doc = link.read().unwrap();
        let location = doc.location().unwrap();
        DocumentExists {
            key: doc.key().clone(),
            path: location.0.clone(),
            location: location.1
        }
    }
}

impl fmt::Display for DocumentExists {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "duplicate document '{}', first defined at {}:{}",
               self.key, self.path, self.location)
    }
}

