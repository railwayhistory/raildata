
use std::fmt;
use std::collections::hash_map::{HashMap, Entry, Values};
use std::sync::{Arc, RwLock};
use ::document::Document;
use ::links::{DocumentLink, Permalink};
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

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn values(&self) -> Values<Key, Permalink> {
        self.documents.values()
    }

    pub fn inner(&self) -> &HashMap<Key, Permalink> {
        &self.documents
    }
}


//------------ PrimaryBuilder ------------------------------------------------

#[derive(Debug, Default)]
pub struct PrimaryBuilder {
    docs: Arc<RwLock<PrimaryIndex>>,
}

impl PrimaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_document(
        &mut self,
        key: Key,
        document: Document
    ) -> Result<(), DocumentExists> {
        let mut docs = self.docs.write().unwrap();
        if let Some(link) = docs.get(&key) {
            {
                let mut doc = link.write().unwrap();
                if doc.is_nonexisting() {
                    *doc = document;
                    return Ok(())
                }
            }
            Err(DocumentExists::from_link(&link))
        }
        else {
            docs.insert(key, Permalink::from_document(document)).unwrap();
            Ok(())
        }
    }

    /// Creates a link to a document identified by a key.
    ///
    /// If a document by this key doesn’t yet exist, creates a placeholder
    /// document and returns a link to that.
    pub fn forge_link(&mut self, key: Key) -> DocumentLink {
        if let Some(link) = self.docs.read().unwrap().get(&key) {
            return link.link()
        }
        let mut map = self.docs.write().unwrap();
        // Someone may have added the document in between ...
        if let Some(link) = map.get(&key) {
            return link.link()
        }
        let link = Permalink::nonexisting(key.clone());
        let res = link.link();
        map.insert(key, link).unwrap();
        res
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

