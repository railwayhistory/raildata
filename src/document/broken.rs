//! A broken document.

use ::types::{Key, Marked};
use super::document::{DocumentType};


//------------ Broken --------------------------------------------------------

/// A document that didn’t actually load properly.
pub struct Broken {
    key: Marked<Key>,
    doc_type: Option<DocumentType>,
}

impl Broken {
    pub fn new(key: Marked<Key>, doc_type: Option<DocumentType>) -> Self {
        Broken { key, doc_type }
    }

    pub fn key(&self) -> &Key {
        self.key.as_value()
    }

    pub fn doc_type(&self) -> Option<DocumentType> {
        self.doc_type
    }
}
