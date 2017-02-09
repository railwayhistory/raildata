//! A collection of heavily cross-referenced, read-only documents.
//!
//! This collection buys easy access to referenced documents with ample
//! opportunity for panics. However, if you follow two simple rules, all
//! should be fine: First, do not follow references until the collection is
//! complete. Second, do not follow references once the collection has been
//! dropped.
//!
//! To make it easier to follow the first rule, there are two types for
//! collections: `CollectionBuilder` is used while loading a collection and,
//! once that is done, is traded in for a `Collection`.

use std::{fmt, ops, ptr};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use ::load::{Error, Source};
use ::documents::{Document, DocumentType};


//------------ Collection ----------------------------------------------------

/// A finalized collection of documents.
///
/// This type represents a collection once loading has finished. All the
/// documents it contains are read-only. They can be accessed only via
/// values of the `DocumentRef` type.
pub struct Collection {
    docs: Vec<Arc<Stored>>
}

impl Collection {
    fn new(docs: Vec<Arc<Stored>>) -> Self {
        Collection {
            docs: docs
        }
    }

    pub fn iter(&self) -> CollectionIter {
        CollectionIter::new(&self.docs)
    }
}

impl<'a> IntoIterator for &'a Collection {
    type Item = DocumentRef;
    type IntoIter = CollectionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CollectionIter::new(&self.docs)
    }
}

unsafe impl Send for Collection { }
unsafe impl Sync for Collection { }


//------------ CollectionIter ------------------------------------------------

pub struct CollectionIter<'a> {
    slice: &'a [Arc<Stored>],
}

impl<'a> CollectionIter<'a> {
    fn new(slice: &'a [Arc<Stored>]) -> Self {
        CollectionIter {
            slice: slice
        }
    }
}

impl<'a> Iterator for CollectionIter<'a> {
    type Item = DocumentRef;

    fn next(&mut self) -> Option<Self::Item> {
        let (first, rest) = match self.slice.split_first() {
            Some(some) => some,
            None => return None,
        };
        self.slice = rest;
        Some(DocumentRef(Arc::downgrade(first)))
    }
}


//------------ CollectionBuilder ---------------------------------------------

#[derive(Clone, Debug)]
pub struct CollectionBuilder(Arc<Mutex<BuilderInner>>);

impl CollectionBuilder {
    pub fn new() -> Self {
        Self::with_errors(Vec::new())
    }

    pub fn with_errors(errors: Vec<Error>) -> Self {
        CollectionBuilder(Arc::new(Mutex::new(BuilderInner::new(errors))))
    }

    pub fn error<E: Into<Error>>(&self, err: E) {
        self.0.lock().unwrap().error(err)
    }

    pub fn str_error<S: Into<Source>>(&self, pos: S, s: &str) {
        self.error(Error::new(pos.into(), String::from(s)))
    }

    pub fn warning<E: Into<Error>>(&self, err: E) {
        self.0.lock().unwrap().error(err)
    }

    pub fn str_warning<S: Into<Source>>(&self, pos: S, s: &str) {
        self.error(Error::new(pos.into(), String::from(s)))
    }

    pub fn ref_doc(&self, key: &str, pos: Source, t: Option<DocumentType>)
                   -> DocumentRef {
        self.0.lock().unwrap().ref_doc(key, pos, t)
    }

    pub fn update_doc(&self, doc: Document, pos: Source)
                      -> Result<(), (Document, Source)> {
        self.0.lock().unwrap().update_doc(doc, pos)
    }

    pub fn broken_doc(&self, key: String, pos: Source) {
        self.0.lock().unwrap().broken_doc(key, pos)
    }

    pub fn finalize(self) -> Result<Result<Collection, Vec<Error>>, Self> {
        Arc::try_unwrap(self.0).map(|mutex| {
            mutex.into_inner().unwrap().finalize()
        }).map_err(CollectionBuilder)
    }
}


//------------ BuilderInner -------------------------------------------------

struct BuilderInner {
    docs: HashMap<String, BuilderValue>,
    errors: Vec<Error>,
}

impl BuilderInner {
    fn new(errors: Vec<Error>) -> Self {
        BuilderInner {
            docs: HashMap::new(),
            errors: errors,
        }
    }

    fn error<E: Into<Error>>(&mut self, err: E) {
        self.errors.push(err.into())
    }

    fn ref_doc(&mut self, key: &str, pos: Source, t: Option<DocumentType>)
                   -> DocumentRef {
        if let Some(value) = self.docs.get_mut(key) {
            return value.ref_doc(pos, t);
        }
        let mut value = BuilderValue::new();
        let res = value.ref_doc(pos, t);
        self.docs.insert(key.to_owned(), value);
        res
    }

    fn update_doc(&mut self, doc: Document, pos: Source)
                      -> Result<(), (Document, Source)> {
        if let Some(value) = self.docs.get_mut(doc.key()) {
            return value.update(doc, pos)
        }
        self.docs.insert(doc.key().to_owned(),
                         BuilderValue::from_doc(doc, pos));
        Ok(())
    }

    fn broken_doc(&mut self, key: String, pos: Source) {
        if let Some(value) = self.docs.get_mut(&key) {
            value.mark_broken(pos);
            return;
        }
        self.docs.insert(key, BuilderValue::broken(pos));
    }

    fn finalize(mut self) -> Result<Collection, Vec<Error>> {
        let mut res = Vec::with_capacity(self.docs.len());
        for (key, value) in self.docs {
            if let Some(doc) = value.into_inner(&key, &mut self.errors) {
                res.push(doc)
            }
        }
        if self.errors.is_empty() {
            Ok(Collection::new(res))
        }
        else {
            Err(self.errors)
        }
    }
}

impl fmt::Debug for BuilderInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CollectionBuilder{{...}}")
    }
}


//------------ BuilderValue --------------------------------------------------

struct BuilderValue {
    /// The document for the value.
    doc: Arc<Stored>,

    /// If true, there is a document but it is broken.
    broken: bool,

    /// The position of the document.
    pos: Option<Source>,

    /// A list of all references.
    /// 
    /// Each element contains the position of the reference and the document
    /// type it requested.
    refs: Vec<(Source, Option<DocumentType>)>,
}

impl BuilderValue {
    fn new() -> Self {
        BuilderValue {
            doc: Arc::new(Stored::new()),
            broken: false,
            pos: None,
            refs: Vec::new(),
        }
    }

    fn broken(pos: Source) -> Self {
        BuilderValue {
            doc: Arc::new(Stored::new()),
            broken: true,
            pos: Some(pos),
            refs: Vec::new(),
        }
    }

    fn from_doc(doc: Document, pos: Source) -> Self {
        BuilderValue {
            doc: Arc::new(Stored::from_doc(doc)),
            broken: false,
            pos: Some(pos),
            refs: Vec::new(),
        }
    }

    fn ref_doc(&mut self, pos: Source, t: Option<DocumentType>)
               -> DocumentRef {
        self.refs.push((pos, t));
        DocumentRef(Arc::downgrade(&self.doc))
    }

    fn update(&mut self, doc: Document, pos: Source)
              -> Result<(), (Document, Source)> {
        if let Some(ref pos) = self.pos {
            return Err((doc, pos.clone()))
        }
        unsafe { self.doc.set(doc) }
        self.pos = Some(pos);
        Ok(())
    }

    fn mark_broken(&mut self, pos: Source) {
        // Quietly ignore the request if we have a document already.
        if self.pos.is_some() {
            self.pos = Some(pos);
            self.broken = true;
        }
    }

    fn into_inner(self, key: &str, errors: &mut Vec<Error>)
                  -> Option<Arc<Stored>> {
        if self.pos.is_none() {
            // We don’t have a document. All references are errors.
            for (pos, _) in self.refs {
                errors.push(Error::new(pos,
                             format!("reference to missing document '{}'",
                                     key)))
            }
            None
        }
        else if !self.broken {
            // We have a document. Check the types of all references.
            let doc_type = unsafe { self.doc.get().doc_type() };
            let mut err = false;
            for (pos, ref_type) in self.refs {
                if let Some(ref_type) = ref_type {
                    if doc_type != ref_type {
                        errors.push(Error::new(pos,
                                     format!("{} reference to {} document '{}'",
                                             ref_type, doc_type, key)));
                        err = true;
                    }
                }
            }
            if err {
                None
            }
            else {
                Some(self.doc)
            }
        }
        else {
            // We have a broken document. Can’t check the references.
            None
        }
    }
}


//------------ DocumentRef ---------------------------------------------------

#[derive(Clone)]
pub struct DocumentRef(Weak<Stored>);

impl DocumentRef {
    pub fn get<U>(&self) -> DocumentGuard<U> {
        DocumentGuard::new(self.0.upgrade().unwrap())
    }
}


//------------ DocumentGuard -------------------------------------------------

pub struct DocumentGuard<U> {
    doc: Arc<Stored>,
    marker: ::std::marker::PhantomData<U>,
}

impl<U> DocumentGuard<U> {
    fn new(doc: Arc<Stored>) -> Self {
        DocumentGuard {
            doc: doc,
            marker: ::std::marker::PhantomData,
        }
    }
}

impl<U> ops::Deref for DocumentGuard<U>
     where Document: AsRef<U> {
    type Target = U;

    fn deref(&self) -> &U {
        unsafe { self.doc.get().as_ref() } 
    }
}


//------------ Stored --------------------------------------------------------

struct Stored(UnsafeCell<*mut Document>);

impl Stored {
    fn new() -> Self {
        Stored(UnsafeCell::new(ptr::null_mut()))
    }

    fn from_doc(doc: Document) -> Self {
        let res = Self::new();
        unsafe { res.set(doc) }
        res
    }

    unsafe fn get(&self) -> &Document {
        assert!(!(*self.0.get()).is_null());
        &**self.0.get()
    }

    unsafe fn set(&self, doc: Document) {
        assert!((*self.0.get()).is_null());
        *self.0.get() = Box::into_raw(Box::new(doc));
    }
}

impl Drop for Stored {
    fn drop(&mut self) {
        unsafe {
            let ptr = *self.0.get();
            if !ptr.is_null() {
                let _  = Box::from_raw(ptr);
            }
        }
    }
}

