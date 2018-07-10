use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ::document::common::DocumentType;
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::Value;
use ::types::{Key, Location, Marked};
use ::types::marked::IntoMarked;
use super::document::{Document, DocumentLink};
use super::update::UpdateStore;


//------------ LoadStore -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct LoadStore(Arc<Mutex<StoreData>>);

#[derive(Clone, Debug)]
struct StoreData {
    documents: Vec<LoadDocument>,
    keys: HashMap<Key, usize>,
}

impl LoadStore {
    pub fn new() -> Self {
        LoadStore(Arc::new(Mutex::new(
            StoreData {
                documents: Vec::new(),
                keys: HashMap::new()
            }
        )))
    }

    pub fn from_yaml(
        &mut self,
        value: Value,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let location = value.location();
        let mut doc = value.into_mapping(report)?;
        let key: Marked<Key> = doc.take("key", self, report)?;
        let doctype = match doc.take("type", self, report) {
            Ok(doctype) => doctype,
            Err(_) => {
                let _ = self.insert_broken(
                    key.into_value(), doc.location(), None, report
                );
                return Ok(())
            }
        };
        match Document::from_yaml(key.clone(), doctype, doc, self, report) {
            Ok(doc) => self.insert(key.into_value(), doc, report),
            Err(_) => {
                self.insert_broken(
                    key.into_value(),
                    location,
                    Some(doctype),
                    report
                )
            }
        }
    }

    pub fn into_update_store(
        self,
        report: &mut StageReporter
    ) -> Result<UpdateStore, Failed> {
        let data = Arc::try_unwrap(self.0).unwrap().into_inner().unwrap();
        let (documents, keys) = (data.documents, data.keys);
        let mut keys: Vec<_> = keys.into_iter().collect();
        keys.sort_by_key(|item| item.1);
        assert!(keys.last().unwrap().1 == keys.len() - 1);

        let mut failed = false;
        for (doc, &(ref key, _)) in documents.iter().zip(keys.iter()) {
            match *doc {
                LoadDocument::Created(_) => { }
                LoadDocument::Linked(ref links) => {
                    for (_, origin) in links {
                        report.error_at(
                            origin.clone(), MissingDocument(key.clone())
                        );
                    }
                    failed = true;
                }
                LoadDocument::Broken(..) => failed = true,
            }
        }

        if failed {
            Err(Failed)
        }
        else {
            Ok(
                UpdateStore::from_documents(
                    documents.into_iter().map(|item| {
                        match item {
                            LoadDocument::Created(doc) => doc,
                            _ => unreachable!()
                        }
                    })
                )
            )
        }
    }
}

impl LoadStore {
    pub fn insert(
        &self,
        key: Key,
        document: Document,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut data = self.0.lock().unwrap();
        match data.keys.get(document.key()).map(|x| x.clone()) {
            Some(pos) => {
                let mut doc = data.documents.get_mut(pos).unwrap();
                match doc {
                    LoadDocument::Created(ref present) => {
                        report.error(
                            DuplicateDocument(
                                present.origin().clone()
                            ).marked(document.location())
                        );
                        return Err(Failed)
                    }
                    LoadDocument::Linked(ref links) => {
                        for link in links {
                            if let Some(doctype) = link.0 {
                                if doctype != document.doctype() {
                                    report.global().error_at(
                                        link.1.clone(),
                                        LinkMismatch {
                                            expected: doctype,
                                            target: document.doctype()
                                        }
                                    )
                                }
                            }
                        }
                        // Add document even if there is broken links, so we
                        // can catch later broken links.
                    }
                    LoadDocument::Broken(_, ref origin) => {
                        report.error(
                            DuplicateDocument(origin.clone())
                                .marked(document.location())
                        );
                        return Err(Failed)
                    }
                }
                *doc = LoadDocument::Created(document);
                Ok(())
            }
            None => {
                let idx = data.documents.len();
                data.keys.insert(key, idx);
                data.documents.push(LoadDocument::Created(document));
                Ok(())
            }
        }
    }

    pub fn insert_broken(
        &self,
        key: Key,
        location: Location,
        doctype: Option<DocumentType>,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut data = self.0.lock().unwrap();
        match data.keys.get(&key).map(|x| x.clone()) {
            Some(pos) => {
                let mut doc = data.documents.get_mut(pos).unwrap();
                match doc {
                    LoadDocument::Created(ref present) => {
                        report.error(
                            DuplicateDocument(
                                present.origin().clone()
                            ).marked(location)
                        );
                        return Err(Failed)
                    }
                    LoadDocument::Linked(ref links) => {
                        if let Some(doctype) = doctype {
                            for link in links {
                                if let Some(linktype) = link.0 {
                                    if linktype != doctype {
                                        report.global().error_at(
                                            link.1.clone(),
                                            LinkMismatch {
                                                expected: linktype,
                                                target: doctype
                                            }
                                        )
                                    }
                                }
                            }
                        }
                        // Add document even if there is broken links, so we
                        // can catch later broken links.
                    }
                    LoadDocument::Broken(_, ref origin) => {
                        report.error(
                            DuplicateDocument(origin.clone())
                                .marked(location)
                        );
                        return Err(Failed)
                    }
                }
                *doc = LoadDocument::Broken(doctype, report.origin(location));
                Ok(())
            }
            None => {
                let idx = data.documents.len();
                data.keys.insert(key, idx);
                data.documents.push(
                    LoadDocument::Broken(doctype, report.origin(location))
                );
                Ok(())
            }
        }
    }

    pub fn forge_link(
        &self,
        key: Marked<Key>,
        linktype: Option<DocumentType>,
        report: &mut PathReporter
    ) -> Result<Marked<DocumentLink>, Failed> {
        let mut data = self.0.lock().unwrap();
        if let Some(pos) = data.keys.get(key.as_ref()).map(|x| x.clone()) {
            let doc = data.documents.get_mut(pos).unwrap();
            match doc {
                LoadDocument::Created(ref present) => {
                    if let Some(linktype) = linktype {
                        if present.doctype() != linktype {
                            report.error(
                                LinkMismatch {
                                    expected: linktype,
                                    target: present.doctype()
                                }.marked(key.location())
                            );
                            return Err(Failed)
                        }
                    }
                }
                LoadDocument::Linked(ref mut links) => {
                    links.push((linktype, report.origin(key.location())));
                }
                LoadDocument::Broken(doctype, _) => {
                    if let (Some(lt), Some(dt)) = (linktype, *doctype) {
                        if lt != dt {
                            report.error(
                                LinkMismatch {
                                    expected: lt,
                                    target: dt
                                }.marked(key.location())
                            );
                            return Err(Failed)
                        }
                    }
                }
            }
            Ok(DocumentLink::new(pos).marked(key.location()))
        }
        else {
            let idx = data.documents.len();
            data.documents.push(
                LoadDocument::Linked(
                    vec![(linktype, report.origin(key.location()))]
                )
            );
            let res = DocumentLink::new(idx).marked(key.location());
            data.keys.insert(key.into_value(), idx);
            Ok(res)
        }
    }
}


//------------ LoadDocument --------------------------------------------------

#[derive(Clone, Debug)]
enum LoadDocument {
    Created(Document),
    Linked(Vec<(Option<DocumentType>, Origin)>),
    Broken(Option<DocumentType>, Origin),
}


//============ Errors ========================================================

#[derive(Clone, Debug, Fail)]
#[fail(display="document already exists, first defied at {}", _0)]
pub struct DuplicateDocument(Origin);

#[derive(Clone, Debug, Fail)]
#[fail(display="link to '{}', expected '{}'", target, expected)]
pub struct LinkMismatch {
    expected: DocumentType,
    target: DocumentType
}

#[derive(Clone, Debug, Fail)]
#[fail(display="link to missing document '{}'", _0)]
pub struct MissingDocument(Key);

