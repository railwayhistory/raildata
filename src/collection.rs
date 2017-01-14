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

use std::{marker, ops, ptr};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Weak};


//------------ Collection ----------------------------------------------------

/// A finalized collection of documents.
///
/// This type represents a collection once loading has finished. All the
/// documents it contains are read-only. They can be accessed only via
/// values of the `DocumentRef` type.
pub struct Collection<T> {
    docs: Vec<Arc<Document<T>>>
}

impl<T> Collection<T> {
    fn new(docs: Vec<Arc<Document<T>>>) -> Self {
        Collection {
            docs: docs
        }
    }

    pub fn iter(&self) -> CollectionIter<T> {
        CollectionIter::new(&self.docs)
    }
}

impl<'a, T: 'a> IntoIterator for &'a Collection<T> {
    type Item = DocumentRef<T>;
    type IntoIter = CollectionIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CollectionIter::new(&self.docs)
    }
}


//------------ CollectionIter ------------------------------------------------

pub struct CollectionIter<'a, T: 'a> {
    slice: &'a [Arc<Document<T>>],
}

impl<'a, T: 'a> CollectionIter<'a, T> {
    fn new(slice: &'a [Arc<Document<T>>]) -> Self {
        CollectionIter {
            slice: slice
        }
    }
}

impl<'a, T: 'a> Iterator for CollectionIter<'a, T> {
    type Item = DocumentRef<T>;

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

pub struct CollectionBuilder<K: Eq + Hash, T, P: Clone> {
    docs: HashMap<K, BuilderValue<T, P>>,

    /// Marker for making this type not be `Send` or `Sync`.
    ///
    /// Because `Rc<T>` is neither of the two, a type built from it will not
    /// be either. The phantom type makes us not actually contain that `Rc`.
    /// (Any other non-sync and non-send type would do, too. `Rc` is just the
    /// first one I could think of.
    marker: marker::PhantomData<::std::rc::Rc<T>>,
}

impl<K: Clone + Eq + Hash, T, P: Clone> CollectionBuilder<K, T, P> {
    pub fn new() -> Self {
        CollectionBuilder {
            docs: HashMap::new(),
            marker: marker::PhantomData,
        }
    }

    pub fn ref_doc(&mut self, key: &K, pos: P) -> DocumentRef<T> {
        if let Some(value) = self.docs.get_mut(key) {
            return value.ref_doc(pos);
        }
        let mut value = BuilderValue::new();
        let res = value.ref_doc(pos);
        self.docs.insert(key.clone(), value);
        res
    }

    pub fn update_doc(&mut self, key: &K, doc: T, pos: P) -> Result<(), P> {
        if let Some(value) = self.docs.get_mut(key) {
            return value.update(doc, pos)
        }
        self.docs.insert(key.clone(), BuilderValue::from_doc(doc, pos));
        Ok(())
    }

    pub fn finalize(self) -> Result<Collection<T>, Vec<(K, Vec<P>)>> {
        let mut res = Vec::with_capacity(self.docs.len());
        let mut err = Vec::new();
        for (key, value) in self.docs {
            match value.into_inner() {
                Ok(doc) => res.push(doc),
                Err(pos) => err.push((key, pos))
            }
        }
        if err.is_empty() {
            Ok(Collection::new(res))
        }
        else {
            Err(err)
        }
    }
}


//------------ BuilderValue --------------------------------------------------

struct BuilderValue<T, P: Clone> {
    /// The document for the value.
    doc: Arc<Document<T>>,

    /// The position information for this value.
    ///
    /// While the document has not been updated, this is `Err(_)` with a
    /// list of the positions of all the handed out references. Once the
    /// document has been updated, it becomes `Ok(_)` with the position of
    /// the defition of the document.
    pos: Result<P, Vec<P>>,
}

impl<T, P: Clone> BuilderValue<T, P> {
    fn new() -> Self {
        BuilderValue {
            doc: Arc::new(Document::new()),
            pos: Err(Vec::new())
        }
    }

    fn from_doc(t: T, pos: P) -> Self {
        BuilderValue {
            doc: Arc::new(Document::from_doc(t)),
            pos: Ok(pos)
        }
    }

    fn ref_doc(&mut self, pos: P) -> DocumentRef<T> {
        if let Err(ref mut list) = self.pos {
            list.push(pos)
        }
        DocumentRef(Arc::downgrade(&self.doc))
    }

    fn update(&mut self, doc: T, pos: P) -> Result<(), P> {
        if let Ok(ref pos) = self.pos {
            return Err(pos.clone())
        }
        unsafe { self.doc.set(doc) }
        self.pos = Ok(pos);
        Ok(())
    }

    fn into_inner(self) -> Result<Arc<Document<T>>, Vec<P>> {
        let (pos, doc) = (self.pos, self.doc);
        pos.map(|_| doc)
    }
}


//------------ DocumentRef ---------------------------------------------------

#[derive(Clone)]
pub struct DocumentRef<T>(Weak<Document<T>>);

impl<T> DocumentRef<T> {
    pub fn get(&self) -> DocumentGuard<T> {
        DocumentGuard(self.0.upgrade().unwrap())
    }
}


//------------ DocumentGuard -------------------------------------------------

pub struct DocumentGuard<T>(Arc<Document<T>>);

impl<T> ops::Deref for DocumentGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.0.get() }
    }
}


//------------ Document ------------------------------------------------------

struct Document<T>(UnsafeCell<*mut T>);

impl<T> Document<T> {
    fn new() -> Self {
        Document(UnsafeCell::new(ptr::null_mut()))
    }

    fn from_doc(t: T) -> Self {
        let res = Self::new();
        unsafe { res.set(t) }
        res
    }

    unsafe fn get(&self) -> &T {
        assert!(!self.0.get().is_null());
        &**self.0.get()
    }

    unsafe fn set(&self, t: T) {
        assert!(self.0.get().is_null());
        *self.0.get() = Box::into_raw(Box::new(t));
    }
}

impl<T> Drop for Document<T> {
    fn drop(&mut self) {
        unsafe { let _  = Box::from_raw(*self.0.get()); }
    }
}
