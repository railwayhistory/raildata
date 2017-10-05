use std::path;
use std::sync::{Arc, Mutex, RwLock};
use std::fs::File;
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use ::documents::document::Document;
use ::documents::index::PrimaryIndex;
use ::documents::types::{Location, Key};
use ::store::{Link, Variant};
use super::construct::Context;
use super::error::{Error, ErrorStore};
use super::path::Path;
use super::read::Utf8Chars;
use super::osm::load_osm_file;
use super::yaml::{Constructor, Loader, Value};


//------------ load_tree -----------------------------------------------------

pub fn load_tree(path: &path::Path) -> Result<PrimaryIndex, ErrorStore> {
    let docs = Arc::new(RwLock::new(PrimaryIndex::new()));
    let errors = Arc::new(Mutex::new(ErrorStore::new()));

    load_facts(path, docs.clone(), errors.clone());
    load_paths(path, docs.clone(), errors.clone());
    
    let docs = Arc::try_unwrap(docs).unwrap().into_inner().unwrap();
    let errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();
    if errors.is_empty() {
        Ok(docs)
    }
    else {
        Err(errors)
    }
}


//------------ load_facts ----------------------------------------------------

fn load_facts(base: &path::Path, docs: Arc<RwLock<PrimaryIndex>>,
              errors: Arc<Mutex<ErrorStore>>) {
    let walk = WalkBuilder::new(base.join("facts"))
                           .types(TypesBuilder::new()
                                               .add_defaults()
                                               .select("yaml")
                                               .build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let mut context = TreeContext::new(docs.clone(), errors.clone());
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = Path::new(path.path());
                context.set_path(path.clone());
                match File::open(&path) {
                    Ok(file) => {
                        if let Err(err) = Loader::new(&mut context)
                                                 .load(Utf8Chars::new(file)) {
                            context.push_error((err, Location::NONE))
                        }
                    }
                    Err(err) => {
                        context.push_error((err, Location::NONE));
                    }
                }
            }
            WalkState::Continue
        })
    })
}


//------------ load_paths ----------------------------------------------------

fn load_paths(base: &path::Path, docs: Arc<RwLock<PrimaryIndex>>,
              errors: Arc<Mutex<ErrorStore>>) {
    let mut types = TypesBuilder::new();
    types.add("osm", "*.osm").unwrap();
    let walk = WalkBuilder::new(base.join("paths"))
                           .types(types.select("osm").build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let mut context = TreeContext::new(docs.clone(), errors.clone());
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = Path::new(path.path());
                context.set_path(path.clone());
                match File::open(&path) {
                    Ok(mut file) => load_osm_file(&mut file, &mut context),
                    Err(err) => {
                        context.push_error((err, Location::NONE))
                    }
                }
            }
            WalkState::Continue
        })
    })
}


//------------ TreeContext ---------------------------------------------------

pub struct TreeContext {
    path: Option<Path>,
    docs: Arc<RwLock<PrimaryIndex>>,
    errors: Arc<Mutex<ErrorStore>>,
}

impl TreeContext {
    fn new(docs: Arc<RwLock<PrimaryIndex>>, errors: Arc<Mutex<ErrorStore>>)
           -> Self {
        TreeContext { path:None , docs, errors }
    }

    fn set_path(&mut self, path: Path) {
        self.path = Some(path)
    }
}


impl<'a> Constructor for &'a mut TreeContext {
    fn construct(&mut self, doc: Value) {
        let loc = doc.location();
        let (doc, key) = match Document::construct(doc,
                                                   self.path.as_ref()
                                                       .unwrap().clone(),
                                                   *self) {
            Some(res) => res,
            None => return
        };
        let err = self.docs.write().unwrap().insert(key, doc);
        if let Err(err) = err {
            self.push_error(Error::new(err, loc))
        }
    }
}

impl TreeContext {
    pub fn insert_document(&mut self, key: Key, document: Document) {
        let err = self.docs.write().unwrap().insert(key, document);
        if let Err(err) = err {
            self.push_error((err, Location::NONE));
        }
    }
}

impl Context for TreeContext {
    fn get_link<T>(&mut self, key: &Key) -> Link<T>
                where T: Variant<Item=Document> {
        self.docs.write().unwrap().get_link(key)
    }

    fn push_error<E: Into<Error>>(&mut self, error: E) {
        self.errors.lock().unwrap().push(self.path.as_ref(), error.into());
    }
}

