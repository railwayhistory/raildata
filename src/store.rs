use std::{borrow, mem};
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound;
use std::sync::{Arc, Mutex};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::document::combined::{Data, Meta, Xrefs};
use crate::document::common::DocumentType;
use crate::load::report::{
    Failed, Origin, PathReporter, Report, Reporter, Stage, StageReporter
};
use crate::load::yaml::{FromYaml, Value};
use crate::types::{IntoMarked, Key, Location, Marked};


//------------ StoreLoader ---------------------------------------------------

/// The store during loading.
#[derive(Debug)]
pub struct StoreLoader {
    data: Mutex<Vec<Option<Data>>>,
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

impl StoreLoader {
    pub fn new() -> Self {
        StoreLoader {
            data: Mutex::new(Vec::new()),
            keys: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_yaml(
        &self,
        value: Value,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let location = value.location();
        let mut doc = value.into_mapping(report)?;
        let key: Marked<Key> = doc.take("key", self, report)?;
        let link = self.get_link(key.as_value());
        let doctype = match doc.take("type", self, report) {
            Ok(doctype) => doctype,
            Err(_) => {
                let _ = self.update_broken(
                    key.as_value(), None, doc.location(), report
                );
                return Ok(())
            }
        };
        match Data::from_yaml(
            key.clone(), doctype, doc, link, self, report
        ) {
            Ok(doc) => self.update(link, doc, report),
            Err(_) => {
                self.update_broken(&key, Some(doctype), location, report)
            }
        }
    }

    pub fn insert(
        &self,
        data: Data,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let link = self.get_link(data.key());
        self.update(link, data, report)
    }

    pub fn insert_broken(
        &self,
        key: Key,
        doctype: Option<DocumentType>,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let _ = self.get_link(&key);
        self.update_broken(&key, doctype, location, report)
    }

    fn get_link(
        &self,
        key: &Key,
    ) -> DocumentLink {
        let mut keys = self.keys.lock().unwrap();

        if let Some(info) = keys.get_mut(key) {
            return info.link
        }

        let link = self.push_none();
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

    fn push_none(&self) -> DocumentLink {
        let mut data = self.data.lock().unwrap();
        let index = data.len();
        data.push(None);
        DocumentLink::from_index(index)
    }

    fn update(
        &self, link: DocumentLink, document: Data, report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut keys = self.keys.lock().unwrap();

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

        let old = mem::replace(
            &mut self.data.lock().unwrap()[link.index],
            Some(document)
        );
        assert!(old.is_none());
        Ok(())
    }

    fn update_broken(
        &self,
        key: &Key,
        doctype: Option<DocumentType>,
        location: Location,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut keys = self.keys.lock().unwrap();

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
        let mut keys = self.keys.lock().unwrap();

        if let Some(info) = keys.get_mut(key.as_ref()) {
            // We don’t check link types here just yet. That happens once
            // when converting to a LibraryMut for all links.
            info.linked_from.push((doctype, report.origin(key.location())));
            return info.link.marked(location)
        }

        let link = self.push_none();
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

    pub fn into_data_store(
        self, report: &mut StageReporter
    ) -> Result<DataStore, Failed> {
        let data = self.data.into_inner().unwrap();
        let docinfo = self.keys.into_inner().unwrap();

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
            Ok(DataStore::new(
                data.into_iter().map(Option::unwrap).collect(),
                keys
            ))
        }
    }
}


//------------ DataStore -----------------------------------------------------

/// A store holding the data portion of documents only.
#[derive(Debug)]
pub struct DataStore {
    data: Vec<Data>,
    keys: BTreeMap<Key, DocumentLink>,
}

impl DataStore {
    fn new(data: Vec<Data>, keys: BTreeMap<Key, DocumentLink>) -> Self {
        DataStore { data, keys }
    }

    pub fn into_xref_store(
        self, report: StageReporter
    ) -> Result<XrefsStore, Failed> {
        XrefsStore::generate(self, report)
    }

    pub fn into_full_store(self) -> Result<FullStore, Report> {
        let report = Reporter::new();

        // Generate the cross references.
        let store = match self.into_xref_store(
            report.clone().stage(Stage::Crossref)
        ) {
            Ok(store) => store,
            Err(_) => return Err(report.unwrap())
        };

        // Generate meta data.
        match store.into_full_store(report.clone().stage(Stage::Meta)) {
            Ok(store) => Ok(store),
            Err(_) => Err(report.unwrap())
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<DocumentLink>
    where Key: borrow::Borrow<Q>, Q: Ord + ?Sized {
        self.keys.get(key).cloned()
    }

    pub fn links(&self) -> impl Iterator<Item = DocumentLink> + '_ {
        self.keys.values().copied()
    }

    pub fn iter(&self) -> impl Iterator<Item=&'_ Data> + '_ {
        self.keys.values().map(move |link| self.resolve(*link))
    }

    pub fn iter_from<T>(
        &self,
        start: &T
    ) -> impl Iterator<Item=&'_ Data> + '_
    where T: Ord + ?Sized, Key: borrow::Borrow<T> {
        self.keys.range((Bound::Included(start), Bound::Unbounded))
            .map(move |link| self.resolve(*link.1))
    }
}

impl LinkTarget<Data> for DataStore {
    fn resolve(&self, link: DocumentLink) -> &Data {
        &self.data[link.index]
    }
}


//------------ XrefsBuilder --------------------------------------------------

#[derive(Debug)]
pub struct XrefsBuilder {
    xrefs: Vec<Xrefs>,
}

impl XrefsBuilder {
    fn new(store: &DataStore) -> Self {
        XrefsBuilder {
            xrefs: store.data.iter().map(Data::default_xrefs).collect()
        }
    }
}

impl LinkTargetMut<Xrefs> for XrefsBuilder {
    fn resolve_mut(&mut self, link: DocumentLink) -> &mut Xrefs {
        &mut self.xrefs[link.index]
    }
}


//------------ XrefsStore -----------------------------------------------------

/// A store holding the data and cross references of documents.
#[derive(Debug)]
pub struct XrefsStore {
    data: Vec<Data>,
    xrefs: Vec<Xrefs>,
    keys: BTreeMap<Key, DocumentLink>,
}

impl XrefsStore {
    fn generate(
        data: DataStore, mut report: StageReporter
    ) -> Result<Self, Failed> {
        let mut xrefs = XrefsBuilder::new(&data);
        let mut ok = true;
        for item in &data.data {
            if item.xrefs(&mut xrefs, &data, &mut report).is_err() {
                ok = false;
            }
        }
        if ok {
            Ok(XrefsStore {
                data: data.data,
                xrefs: xrefs.xrefs,
                keys: data.keys,
            })
        }
        else {
            Err(Failed)
        }
    }

    pub fn into_full_store(
        self, report: StageReporter
    ) -> Result<FullStore, Failed> {
        FullStore::generate(self, report)
    }

    pub fn links(&self) -> impl Iterator<Item = DocumentLink> + '_ {
        self.keys.values().copied()
    }
}

impl LinkTarget<Data> for XrefsStore {
    fn resolve(&self, link: DocumentLink) -> &Data {
        &self.data[link.index]
    }
}

impl LinkTarget<Xrefs> for XrefsStore {
    fn resolve(&self, link: DocumentLink) -> &Xrefs {
        &self.xrefs[link.index]
    }
}


//------------ FullStore -----------------------------------------------------

/// The store with both the data and the meta data.
#[derive(Debug)]
pub struct FullStore {
    data: Vec<Data>,
    xrefs: Vec<Xrefs>,
    meta: Vec<Meta>,
    keys: BTreeMap<Key, DocumentLink>,
}

impl FullStore {
    fn generate(
        store: XrefsStore, mut report: StageReporter
    ) -> Result<Self, Failed> {
        let mut meta = Vec::with_capacity(store.data.len());
        let mut ok = true;
        for data in &store.data {
            match Meta::generate(data, &store, &mut report) {
                Ok(res) => {
                    if ok {
                        meta.push(res)
                    }
                }
                Err(_) => { ok = false; }
            }
        }
        if ok {
            Ok(FullStore {
                data: store.data,
                xrefs: store.xrefs,
                meta,
                keys: store.keys,
            })
        }
        else {
            Err(Failed)
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<DocumentLink>
    where Key: borrow::Borrow<Q>, Q: Ord + ?Sized {
        self.keys.get(key).cloned()
    }

    pub fn links(&self) -> impl Iterator<Item = DocumentLink> + '_ {
        self.keys.values().copied()
    }

    pub fn iter_from<T>(
        &self,
        start: &T
    ) -> impl Iterator<Item = DocumentLink> + '_
    where T: Ord + ?Sized, Key: borrow::Borrow<T> {
        self.keys.range(
            (Bound::Included(start), Bound::Unbounded)
        ).map(|item| *item.1)
    }
}

impl LinkTarget<Data> for FullStore {
    fn resolve(&self, link: DocumentLink) -> &Data {
        &self.data[link.index]
    }
}

impl LinkTarget<Xrefs> for FullStore {
    fn resolve(&self, link: DocumentLink) -> &Xrefs {
        &self.xrefs[link.index]
    }
}

impl LinkTarget<Meta> for FullStore {
    fn resolve(&self, link: DocumentLink) -> &Meta {
        &self.meta[link.index]
    }
}


//------------ DocumentLink --------------------------------------------------

/// A link to another document.
///
/// Links remain stable between all stores derived from the same
/// [`StoreLoader`] instance.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct DocumentLink {
    index: usize,
}

impl DocumentLink {
    fn from_index(index: usize) -> Self {
        DocumentLink { index }
    }

    pub fn data(self, store: &impl LinkTarget<Data>) -> &Data {
        store.resolve(self)
    }

    pub fn xrefs(self, store: &impl LinkTarget<Xrefs>) -> &Xrefs {
        store.resolve(self)
    }

    pub fn xrefs_mut(
        self, store: &mut impl LinkTargetMut<Xrefs>
    ) -> &mut Xrefs {
        store.resolve_mut(self)
    }

    pub fn meta(self, store: &impl LinkTarget<Meta>) -> &Meta {
        store.resolve(self)
    }
}

impl FromYaml<StoreLoader> for Marked<DocumentLink> {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Ok(context.build_link(
            Marked::from_yaml(value, context, report)?, None, report
        ))
    }
}


//------------ LinkTarget ----------------------------------------------------

pub trait LinkTarget<T> {
    fn resolve(&self, link: DocumentLink) -> &T;
}

impl<T, U: LinkTarget<T>> LinkTarget<T> for Arc<U> {
    fn resolve(&self, link: DocumentLink) -> &T {
        self.as_ref().resolve(link)
    }
}

pub trait LinkTargetMut<T> {
    fn resolve_mut(&mut self, link: DocumentLink) -> &mut T;
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

