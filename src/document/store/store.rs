use std::ops;
use super::document::Document;


//------------ Store ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Store {
    documents: Vec<Document>,
}

impl Store {
    pub fn from_documents<I: Iterator<Item=Document>>(iter: I) -> Self {
        Store { documents: iter.collect() }
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }
}


//------------ Stored --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Stored<'a, T: 'a> {
    item: &'a T,
    store: &'a Store,
}

impl<'a, T: 'a> Stored<'a, T> {
    pub fn stored_document(&self, pos: usize) -> &'a Document {
        self.store.documents.get(pos).unwrap()
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

