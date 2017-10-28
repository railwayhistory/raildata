//! Constructing data from its YAML represenation.

use std::sync::{Arc, RwLock};
use ::document::Document;
use ::index::{PrimaryIndex, DocumentExists};
use ::links::{DocumentLink, Permalink};
use ::types::Key;
use ::types::list::ListError;
use super::error::{Error, SharedErrorStore};
use super::path::Path;
use super::yaml::{Value, Constructor};


//------------ ConstructContext ----------------------------------------------

/// A type collecting everything needed during construction.
pub struct ConstructContext {
    path: Path,
    docs: Arc<RwLock<PrimaryIndex>>,
    errors: SharedErrorStore,
}

impl ConstructContext {
    pub fn new(path: Path, docs: Arc<RwLock<PrimaryIndex>>,
               errors: SharedErrorStore)
               -> Self {
        ConstructContext { path, docs, errors }
    }
}

impl ConstructContext {
    /// Appends an error to the context’s error store.
    pub fn push_error<E: Into<Error>>(&mut self, error: E) {
        self.errors.push(Some(&self.path), error)
    }

    pub fn insert_document(&mut self, key: Key, document: Document) {
        let err = {
            let mut docs = self.docs.write().unwrap();
            if let Some(link) = docs.get(&key) {
                {
                    let mut doc = link.write().unwrap();
                    if doc.is_nonexisting() {
                        *doc = document;
                        return;
                    }
                }
                DocumentExists::from_link(&link)
            }
            else {
                docs.insert(key, Permalink::from_document(document)).unwrap();
                return;
            }
        };
        self.push_error((err, document.location().unwrap().1));
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

    /// Provides a reference to the path we are currently working on.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn ok<T, E: Into<Error>>(&mut self, res: Result<T, E>)
                                 -> Result<T, Failed> {
        match res {
            Ok(some) => Ok(some),
            Err(err) => {
                self.push_error(err);
                Err(Failed)
            }
        }
    }
}

impl<'a> Constructor for &'a mut ConstructContext {
    fn construct(&mut self, doc: Value) {
        let (doc, key) = match Document::construct(doc, self) {
            Ok(doc) => doc,
            Err(_) => return
        };
        self.insert_document(key, doc);
    }
}


//------------ Constructable -------------------------------------------------

/// A trait for types that can be constructed from a YAML value.
pub trait Constructable: Sized {
    /// Constructs a value of the type from a YAML value.
    ///
    /// Helpful methods for construction are available through `context`. If
    /// construction fails, an error should be added via
    /// `context.push_error()` and `Err(Failed)` returned.
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed>;
}

impl<T: Constructable> Constructable for Option<T> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        if value.is_null() {
            Ok(None)
        }
        else {
            T::construct(value, context).map(Some)
        }
    }
}

impl<T: Constructable> Constructable for Vec<T> {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let location = value.location();
        Ok(match value.try_into_sequence() {
            Ok(seq) => {
                if seq.is_empty() {
                    context.push_error((ListError::Empty, location));
                    return Err(Failed)
                }
                else {
                    let mut res = Vec::with_capacity(seq.len());
                    let mut err = false;
                    for item in seq.into_value() {
                        if let Ok(item) = T::construct(item, context) {
                            res.push(item)
                        }
                        else {
                            err = true
                        }
                    }
                    if err { 
                        return Err(Failed)
                    }
                    res
                }
            }
            Err(value) => {
                let value = T::construct(value, context)?;
                vec![value]
            }
        })
    }
}

impl Constructable for f64 {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        value.into_float(context).map(|v| v.into_value())
    }
}


//------------ Failed --------------------------------------------------------

/// Construction has failed.
///
/// This is only a marker type. Information about the failure has been added
/// to the `ConstructContext`.
#[derive(Clone, Copy, Debug)]
pub struct Failed;

