
use std::{fmt, mem};
use std::collections::hash_map::{HashMap, Entry};
use ::store::{ItemError, Link, PermalinkError, Variant};
use super::document::{Document, DocumentLink, DocumentStore};
use super::types::Key;


//------------ KeyIndex ------------------------------------------------------

pub struct KeyIndex {
    links: HashMap<Key, DocumentLink>,
}

impl KeyIndex {
    pub fn new() -> Self {
        KeyIndex { links: HashMap::new() }
    }

    pub fn get(&self, key: &Key) -> Option<DocumentLink> {
        self.links.get(key).map(Clone::clone)
    }

    pub fn get_link<T>(&self, key: &Key) -> Result<Option<Link<T>>,
                                                   PermalinkError<T::Err>>
                    where T: Variant<Item=Document> {
        match self.links.get(key) {
            None => Ok(None),
            Some(value) => Ok(Some(value.convert()?.downgrade()))
        }
    }
}


//------------ PrimaryIndex --------------------------------------------------

pub struct PrimaryIndex {
    store: DocumentStore,
    links: HashMap<Key, DocumentLink>,
}

impl PrimaryIndex {
    pub fn new() -> Self {
        PrimaryIndex { store: DocumentStore::new(), links: HashMap::new() }
    }

    /// Inserts a document into the index.
    ///
    /// This takes an owned key (even though it could clone it from the
    /// document’s own key), since we keep a clone around during document
    /// creation anyway to simplify error handling (and we keep that around
    /// because we need a copy for inserting into the index, anyway – so
    /// this works out great).
    pub fn insert(&mut self, key: Key, mut doc: Document)
                  -> Result<(), KeyInsertError> {
        match self.links.entry(key) {
            Entry::Occupied(mut entry) => {
                if entry.get().get()?.is_nonexisting() {
                    mem::swap(entry.get_mut().get_mut()?.as_mut(), &mut doc);
                    Ok(())
                }
                else {
                    Err(KeyInsertError::KeyExists(entry.get().clone()))
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(self.store.insert(doc));
                Ok(())
            }
        }
    }

    pub fn get_link<T>(&mut self, key: &Key) -> Link<T>
                    where T: Variant<Item=Document> {
        if let Some(link) = self.links.get(key) {
            return link.force_convert().downgrade()
        }
        let link = self.store.insert(Document::nonexisting(key.clone()));
        let res = link.force_convert().downgrade();
        let old = self.links.insert(key.clone(), link);
        assert!(old.is_none());
        res
    }

    pub fn finalize(self) -> (DocumentStore, KeyIndex) {
        (self.store, KeyIndex { links: self.links })
    }
}

impl fmt::Debug for PrimaryIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("PrimaryIndex{...}")
    }
}


//------------ KeyInsertError ------------------------------------------------

pub enum KeyInsertError {
    Poisoned,
    KeyExists(DocumentLink),
}

impl fmt::Display for KeyInsertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KeyInsertError::Poisoned
                => f.write_str("internal error: poisoned lock"),
            KeyInsertError::KeyExists(ref link) => {
                let link = link.get().unwrap();
                let (path, loc) = link.location();
                write!(f, "duplicate key '{}'; first at {}:{}",
                       link.key(), path, loc)
            }
        }
    }
}

impl From<ItemError> for KeyInsertError {
    fn from(_: ItemError) -> KeyInsertError {
        KeyInsertError::Poisoned
    }
}

