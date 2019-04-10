use std::{borrow, ops};
use std::collections::BTreeMap;
use std::ops::Bound;
use ::document::common::DocumentType;
use ::types::key::Key;
use super::document::{Document, DocumentLink, StoredDocument};


//------------ Store ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Store {
    documents: Vec<Document>,
    index: BTreeMap<Key, (DocumentLink, DocumentType)>,
}

impl Store {
    pub fn from_documents<I: Iterator<Item=Document>>(iter: I) -> Self {
        let mut documents = Vec::new();
        let mut index = BTreeMap::new();

        for (pos, document) in iter.enumerate() {
            index.insert(
                document.key().clone(),
                (DocumentLink::new(pos), document.doctype())
            );
            documents.push(document)
        }

        Store { documents, index }
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn resolve(&self, link: DocumentLink) -> StoredDocument {
        StoredDocument::new(self.documents.get(link.pos()).unwrap(), self)
    }

    pub fn iter<'a>(
        &'a self
    ) -> impl Iterator<Item=StoredDocument<'a>> + 'a {
        self.index.values().map(move |item| self.resolve(item.0.clone()))
    }

    pub fn iter_from<'a, T>(
        &'a self,
        start: &T
    ) -> impl Iterator<Item=StoredDocument<'a>> + 'a
    where T: Ord + ?Sized, Key: borrow::Borrow<T> {
        self.index.range((Bound::Included(start), Bound::Unbounded))
            .map(move |item| self.resolve((item.1).0.clone()))
    }

    pub fn get<K: AsRef<Key>>(&self, key: K) -> Option<DocumentLink> {
        self.index.get(key.as_ref()).map(|item| item.0.clone())
    }
}


//------------ Stored --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Stored<'a, T: 'a> {
    item: &'a T,
    store: &'a Store,
}

impl<'a, T: 'a> Stored<'a, T> {
    pub(crate) fn new(item: &'a T, store: &'a Store) -> Self {
        Stored { item, store }
    }

    pub fn store(&self) -> &'a Store {
        self.store
    }

    pub fn access(&self) -> &'a T {
        self.item
    }

    pub fn map<F, U: 'a>(&self, op: F) -> Stored<'a, U>
    where F: FnOnce(&'a T) -> &'a U {
        Stored { item: op(self.item), store: self.store }
    }

    pub fn map_opt<F, U: 'a>(&self, op: F) -> Option<Stored<'a, U>>
    where F: FnOnce(&'a T) -> Option<&'a U> {
        op(self.item).map(|item| {
            Stored { item, store: self.store }
        })
    }

    pub fn wrap<F, U>(&self, op: F) -> ForStored<'a, U>
    where F: FnOnce(&'a T) -> U {
        ForStored { item: op(self.item), store: self.store }
    }
}

impl<'a, T: 'a> ops::Deref for Stored<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}


//------------ ForStored -----------------------------------------------------

pub struct ForStored<'a, T> {
    item: T,
    store: &'a Store,
}

impl<'a, T> ForStored<'a, T> {
    pub fn as_stored<U: 'a>(&self, x: &'a U) -> Stored<'a, U> {
        Stored {
            item: x,
            store: self.store
        }
    }
}

impl<'a, T: 'a> AsRef<T> for ForStored<'a, T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<'a, T: 'a> AsMut<T> for ForStored<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<'a, T: 'a> ops::Deref for ForStored<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item
    }
}

