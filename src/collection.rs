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

use std::{fmt, marker, ops, ptr};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use ::load::{ErrorGatherer, Source};
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

pub struct CollectionBuilder {
    docs: HashMap<String, BuilderValue>,

    /// Marker for making this type not be `Send` or `Sync`.
    ///
    /// Because `Rc` is neither of the two, a type built from it will not
    /// be either. The phantom type makes us not actually contain that `Rc`.
    /// (Any other non-sync and non-send type would do, too. `Rc` is just the
    /// first one I could think of.
    marker: marker::PhantomData<::std::rc::Rc<Document>>,
}

impl CollectionBuilder {
    pub fn new() -> Self {
        CollectionBuilder {
            docs: HashMap::new(),
            marker: marker::PhantomData,
        }
    }

    pub fn ref_doc(&mut self, key: &str, pos: Source, t: DocumentType)
                   -> DocumentRef {
        if let Some(value) = self.docs.get_mut(key) {
            return value.ref_doc(pos, t);
        }
        let mut value = BuilderValue::new();
        let res = value.ref_doc(pos, t);
        self.docs.insert(key.to_owned(), value);
        res
    }

    pub fn update_doc(&mut self, doc: Document, pos: Source)
                      -> Result<(), (Document, Source)> {
        if let Some(value) = self.docs.get_mut(doc.key()) {
            return value.update(doc, pos)
        }
        self.docs.insert(doc.key().to_owned(),
                         BuilderValue::from_doc(doc, pos));
        Ok(())
    }

    pub fn finalize(self, errors: &ErrorGatherer) -> Option<Collection> {
        let mut res = Vec::with_capacity(self.docs.len());
        let mut err = false;
        for (key, value) in self.docs {
            match value.into_inner(&key, errors) {
                Some(doc) => res.push(doc),
                None => err = true,
            }
        }
        if !err {
            Some(Collection::new(res))
        }
        else {
            None
        }
    }
}

impl fmt::Debug for CollectionBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CollectionBuilder{{...}}")
    }
}


//------------ BuilderValue --------------------------------------------------

struct BuilderValue {
    /// The document for the value.
    doc: Arc<Stored>,

    /// The position of the document.
    pos: Option<Source>,

    /// A list of all references.
    /// 
    /// Each element contains the position of the reference and the document
    /// type it requested.
    refs: Vec<(Source, DocumentType)>,
}

impl BuilderValue {
    fn new() -> Self {
        BuilderValue {
            doc: Arc::new(Stored::new()),
            pos: None,
            refs: Vec::new(),
        }
    }

    fn from_doc(doc: Document, pos: Source) -> Self {
        BuilderValue {
            doc: Arc::new(Stored::from_doc(doc)),
            pos: Some(pos),
            refs: Vec::new(),
        }
    }

    fn ref_doc(&mut self, pos: Source, t: DocumentType) -> DocumentRef {
        self.refs.push((pos, t));
        DocumentRef(Arc::downgrade(&self.doc))
    }

    fn update(&mut self, doc: Document, pos: Source)
              -> Result<(), (Document, Source)> {
        if self.pos.is_some() {
            return Err((doc, pos.clone()))
        }
        unsafe { self.doc.set(doc) }
        self.pos = Some(pos);
        Ok(())
    }

    fn into_inner(self, key: &str, errors: &ErrorGatherer)
                  -> Option<Arc<Stored>> {
        if self.pos.is_none() {
            // We don’t have a document. All references are errors.
            for (pos, _) in self.refs {
                errors.add((pos,
                            format!("reference to missing document '{}'",
                                    key)))
            }
            None
        }
        else {
            // We have a document. Check the types of all references.
            let doc_type = unsafe { self.doc.get() .doc_type() };
            let mut err = false;
            for (pos, ref_type) in self.refs {
                if doc_type != ref_type {
                    errors.add((pos,
                                format!("{} reference to {} document '{}'",
                                        ref_type, doc_type, key)));
                    err = true;
                }
            }
            if err {
                None
            }
            else {
                Some(self.doc)
            }
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
        unsafe { let _  = Box::from_raw(*self.0.get()); }
    }
}

