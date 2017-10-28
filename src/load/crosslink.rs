//! Crosslinking documents.

use std::sync::{Arc, RwLock};
use ::index::PrimaryIndex;
use ::links::Permalink;
use ::types::Key;
use super::error::{Error, SharedErrorStore};
use super::path::Path;


//------------ CrosslinkContext ----------------------------------------------

pub struct CrosslinkContext {
    docs: Arc<RwLock<PrimaryIndex>>,
    errors: SharedErrorStore,
}

impl CrosslinkContext {
    pub fn new(docs: Arc<RwLock<PrimaryIndex>>, errors: SharedErrorStore)
               -> Self {
        CrosslinkContext { docs, errors }
    }
}

impl CrosslinkContext {
    /// Appends an error to the context’s error store.
    pub fn push_error<E: Into<Error>>(&mut self, path: Option<&Path>,
                                      error: E) {
        self.errors.push(path, error)
    }

    /// Returns a permalink for accessing the specified document.
    /// 
    /// # Panics
    ///
    /// The method panics if a document with that key doesn’t exist. Since
    /// existance of all documents should have been checked before
    /// crosslinking this is fine.
    pub fn get_link(&self, key: &Key) -> Permalink {
        self.docs.read().unwrap().get(key).unwrap()
    }


}
