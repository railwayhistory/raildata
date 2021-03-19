
use std::{borrow, io};
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound;
use std::sync::{Arc, Mutex};
use derive_more::Display;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::document::{Document, DocumentLink};
use crate::document::common::DocumentType;
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::Value;
use crate::store::{
    Store, StoreBuilder, StoreMut, StoreReadGuard, StoreWriteGuard
};
use crate::types::{IntoMarked, Key, Location, Marked};


//------------ LibraryBuilder ------------------------------------------------

/// Initial builder for the library.
///
/// This is used during stage one. You can only load documents and establish
/// links between them.
#[derive(Clone, Debug)]
pub struct LibraryBuilder(Arc<BuilderData>);

#[derive(Debug)]
struct BuilderData {
    store: StoreBuilder<Document>,
    keys: Mutex<HashMap<Key, DocumentInfo>>,
}

#[derive(Debug)]
struct DocumentInfo {
    /// A link to this document.
    link: DocumentLink,

    /// The document type, if already known.
    doctype: Option<DocumentType>,

    /// The document origin, if already known.
    ///
    /// This also serves as an indication whether we have an actual document
    /// for this key already.
    origin: Option<Origin>,

    /// A list of who linked to this document.
    ///
    /// The entries are the origin and optionally if they requested a certain
    /// type.
    linked_from: Vec<(Option<DocumentType>, Origin)>,

    /// Is this a broken document?
    broken: bool,
}

impl LibraryBuilder {
    pub fn new() -> Self {
        LibraryBuilder(Arc::new(
            BuilderData {
                store: StoreBuilder::new(),
                keys: Mutex::new(HashMap::new())
            }
        ))
    }

    pub fn from_yaml(
        &self,
        value: Value,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let location = value.location();
        let mut doc = value.into_mapping(report)?;
        let key: Marked<Key> = doc.take("key", self, report)?;
        let link = self._get_link(key.as_value());
        let doctype = match doc.take("type", self, report) {
            Ok(doctype) => doctype,
            Err(_) => {
                let _ = self._update_broken(
                    key.as_value(), None, doc.location(), report
                );
                return Ok(())
            }
        };
        match Document::from_yaml(
            key.clone(), doctype, doc, link, self, report
        ) {
            Ok(doc) => self._update(link, doc, report),
            Err(_) => {
                self._update_broken(&key, Some(doctype), location, report)
            }
        }
    }

    pub fn insert(
        &self,
        document: Document,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let link = self._get_link(document.key());
        self._update(link, document, report)
    }

    pub fn insert_broken(
        &self,
        key: Key,
        doctype: Option<DocumentType>,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let _ = self._get_link(&key);
        self._update_broken(&key, doctype, location, report)
    }

    fn _get_link(
        &self,
        key: &Key,
    ) -> DocumentLink {
        let mut keys = self.0.keys.lock().unwrap();

        if let Some(info) = keys.get_mut(key) {
            return info.link
        }

        let link = self.0.store.push(None).into();
        keys.insert(
            key.clone(),
            DocumentInfo {
                link,
                doctype: None,
                origin: None,
                linked_from: Vec::new(),
                broken: false,
            }
        );
        link
    }

    fn _update(
        &self, link: DocumentLink, document: Document, report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut keys = self.0.keys.lock().unwrap();

        let info = keys.get_mut(document.key()).unwrap();

        if info.origin.is_some() {
            report.error(
                DuplicateDocument(
                    info.origin.clone().unwrap()
                ).marked(document.origin().location())
            );
            return Err(Failed);
        }

        info.doctype = Some(document.doctype());
        info.origin = Some(document.origin().clone());
        info.broken = false;

        let res = self.0.store.update(link.into(), document);
        assert!(res.is_none());
        Ok(())
    }
        

    fn _update_broken(
        &self,
        key: &Key,
        doctype: Option<DocumentType>,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut keys = self.0.keys.lock().unwrap();

        let info = keys.get_mut(key).unwrap();

        if info.origin.is_some() {
            report.error(
                DuplicateDocument(
                    info.origin.clone().unwrap()
                ).marked(location)
            );
            return Err(Failed);
        }
            
        info.doctype = doctype;
        info.origin = Some(report.origin(location));
        info.broken = true;
        Ok(())
    }

    pub fn build_link(
        &self,
        key: Marked<Key>,
        doctype: Option<DocumentType>,
        report: &mut PathReporter
    ) -> Marked<DocumentLink> {
        let location = key.location();
        let mut keys = self.0.keys.lock().unwrap();

        if let Some(info) = keys.get_mut(key.as_ref()) {
            // We don’t check link types here just yet. That happens once
            // when converting to a LibraryMut for all links.
            info.linked_from.push((doctype, report.origin(key.location())));
            return info.link.marked(location)
        }

        let link = self.0.store.push(None).into();
        keys.insert(
            key.into_value(),
            DocumentInfo {
                link,
                doctype: None,
                origin: None,
                linked_from: vec![(doctype, report.origin(location))],
                broken: false
            }
        );
        link.marked(location)
    }

    pub fn into_library_mut(
        self,
        report: &mut StageReporter
    ) -> Result<LibraryMut, Failed> {
        let data = Arc::try_unwrap(self.0).unwrap();
        let store = data.store;
        let docinfo = data.keys.into_inner().unwrap();

        let mut failed = false;
        let mut keys = BTreeMap::new();
        for (key, info) in docinfo {
            // If the document is broken, there was an error before and we
            // don’t need to worry about it. But, we said failed just so we
            // stop.
            if info.broken {
                failed = true;
            }

            // If origin is None, we have a missing document. All links are
            // errors.
            if info.origin.is_none() {
                for &(_, ref origin) in &info.linked_from {
                    report.error_at(
                        origin.clone(), MissingDocument(key.clone())
                    );
                }
                failed = true;
            }

            // All links that have a differing doctype are bad.
            if let Some(target) = info.doctype {
                for (expected, origin) in info.linked_from {
                    if let Some(expected) = expected {
                        if expected != target {
                            report.error_at(
                                origin,
                                LinkMismatch { expected, target }
                            );
                            failed = true;
                        }
                    }
                }
            }

            if !failed {
                keys.insert(key, info.link);
            }
        }
        if failed {
            Err(Failed)
        }
        else {
            Ok(LibraryMut(Arc::new(MutData {
                store: store.into(),
                keys
            })))
        }
    }
}


//------------ LibraryMut ----------------------------------------------------

/// A library that allows mutating its content.
///
/// This is used during stage two. You can’t add any documents anymore, but
/// you can get mutable access to them.
pub struct LibraryMut(Arc<MutData>);

#[derive(Debug)]
struct MutData {
    /// The documents.
    store: StoreMut<Document>,
    keys: BTreeMap<Key, DocumentLink>,
}

impl LibraryMut {
    pub fn into_library(self) -> Library {
        let data = Arc::try_unwrap(self.0).unwrap();
        Library(Arc::new(
            Data {
                store: data.store.into(),
                keys: data.keys,
            }
        ))
    }

    pub fn iter(&self) -> impl Iterator<Item = DocumentLink> {
        self.0.store.iter().map(Into::into)
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item=DocumentLink> {
        self.0.store.par_iter().map(Into::into)
    }

    pub fn read(&self) -> LibraryReadGuard {
        LibraryReadGuard { guard: self.0.store.read() }
    }

    pub fn write(&self) -> LibraryWriteGuard {
        LibraryWriteGuard { guard: self.0.store.write() }
    }
}


//------------ LibraryReadGuard ----------------------------------------------

pub struct LibraryReadGuard<'a> {
    guard: StoreReadGuard<'a, Document>,
}

impl<'a> LinkTarget for LibraryReadGuard<'a> {
    fn resolve(&self, link: DocumentLink) -> &Document {
        self.guard.resolve(link.into())
    }
}


//------------ LibraryWriteGuard ---------------------------------------------

pub struct LibraryWriteGuard<'a> {
    guard: StoreWriteGuard<'a, Document>,
}

impl<'a> LinkTargetMut for LibraryWriteGuard<'a> {
    fn resolve_mut(&mut self, link: DocumentLink) -> &mut Document {
        self.guard.resolve_mut(link.into())
    }
}


//------------ Library -------------------------------------------------------

/// The real library.
///
/// It cannot be changed at all. Libraries are forever.
#[derive(Clone)]
pub struct Library(Arc<Data>);

#[derive(Serialize, Deserialize)]
struct Data {
    store: Store<Document>,
    keys: BTreeMap<Key, DocumentLink>,
}

impl Library {
    pub fn len(&self) -> usize {
        self.0.store.len()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<DocumentLink>
    where Key: borrow::Borrow<Q>, Q: Ord + ?Sized {
        self.0.keys.get(key).cloned()
    }

    pub fn links<'s>(&'s self) -> impl Iterator<Item = DocumentLink> + 's {
        self.0.keys.values().copied()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a Document> + 'a {
        self.0.keys.values().map(move |link| self.resolve(*link))
    }

    pub fn iter_from<'a, T>(
        &'a self,
        start: &T
    ) -> impl Iterator<Item=&'a Document> + 'a
    where T: Ord + ?Sized, Key: borrow::Borrow<T> {
        self.0.keys.range((Bound::Included(start), Bound::Unbounded))
            .map(move |link| self.resolve(*link.1))
    }

    pub fn store(&self) -> &Store<Document> {
        &self.0.store
    }

    pub fn write<W: io::Write>(
        &self, writer: W
    ) -> Result<(), bincode::Error> {
        bincode::serialize_into(writer, self.0.as_ref())
    }

    pub fn read<R: io::Read>(
        reader: R
    ) -> Result<Self, bincode::Error> {
        bincode::deserialize_from(reader)
            .map(|data| Library(Arc::new(data)))
    }
}

impl LinkTarget for Library {
    fn resolve(&self, link: DocumentLink) -> &Document {
        self.0.store.resolve(link.into())
    }
}


//------------ LinkTarget, LinkTargetMut -------------------------------------

pub trait LinkTarget {
    fn resolve(&self, link: DocumentLink) -> &Document;
}

pub trait LinkTargetMut {
    fn resolve_mut(&mut self, link: DocumentLink) -> &mut Document;
}


//============ Errors ========================================================

#[derive(Clone, Debug, Display)]
#[display(fmt="document already exists, first defined at {}", _0)]
pub struct DuplicateDocument(Origin);

#[derive(Clone, Debug, Display)]
#[display(fmt="link to '{}', expected '{}'", target, expected)]
pub struct LinkMismatch {
    expected: DocumentType,
    target: DocumentType
}

#[derive(Clone, Debug, Display)]
#[display(fmt="link to missing document '{}'", _0)]
pub struct MissingDocument(Key);


