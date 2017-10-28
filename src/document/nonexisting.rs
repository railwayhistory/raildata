//! A not-existing document.

use std::fmt;
use ::load::error::SharedErrorStore;
use ::load::path::Path;
use ::types::{Key, Location};
use super::document::{DocumentType, VariantError};


//------------ Nonexisting ---------------------------------------------------

/// A document that doesn’t actually exist.
pub struct Nonexisting {
    key: Key,
    links: Vec<Link>,
}

impl Nonexisting {
    pub fn new(key: Key) -> Self {
        Nonexisting { key, links: Vec::new() }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn add_link(&mut self, path: Path, location: Location,
                    doctype: Option<DocumentType>) {
        self.links.push(Link { path, location, doctype })
    }

    pub fn convert(self, doctype: DocumentType, errors: &SharedErrorStore) {
        for link in self.links {
            if let Some(linktype) = link.doctype {
                if linktype != doctype {
                    errors.push(Some(&link.path),
                                (VariantError::new(linktype, Some(doctype)),
                                 link.location));
                }
            }
        }
    }

    pub fn complain(&self, errors: &SharedErrorStore) {
        for link in &self.links {
            errors.push(Some(&link.path),
                        (MissingDocument(self.key.clone()),
                         link.location))
        }
    }
}


//------------ Link ----------------------------------------------------------

struct Link {
    path: Path,
    location: Location,
    doctype: Option<DocumentType>,
}


//------------ MissingDocument -----------------------------------------------

#[derive(Clone, Debug)]
struct MissingDocument(Key);

impl fmt::Display for MissingDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reference to non-existing document '{}'", &self.0)
    }
}

