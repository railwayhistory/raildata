use std::path;
use std::sync::{Arc, RwLock};
use std::fs::File;
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use ::index::PrimaryIndex;
use ::types::Location;
use super::construct::ConstructContext;
use super::error::{ErrorStore, SharedErrorStore};
use super::path::Path;
use super::read::Utf8Chars;
use super::osm::load_osm_file;
use super::yaml::Loader;


//------------ load_tree -----------------------------------------------------

pub fn load_tree(path: &path::Path) -> Result<PrimaryIndex, ErrorStore> {
    let docs = Arc::new(RwLock::new(PrimaryIndex::new()));
    let errors = SharedErrorStore::new();

    load_facts(path, docs.clone(), errors.clone());
    load_paths(path, docs.clone(), errors.clone());
 
    let docs = Arc::try_unwrap(docs).unwrap().into_inner().unwrap();
    for value in docs.values() {
        if let Some(doc) = value.read().unwrap().as_nonexisting() {
            doc.complain(&errors)
        }
    }
    let errors = errors.try_unwrap().unwrap();
    if errors.is_empty() {
        Ok(docs)
    }
    else {
        Err(errors)
    }
}


//------------ load_facts ----------------------------------------------------

fn load_facts(base: &path::Path, docs: Arc<RwLock<PrimaryIndex>>,
              errors: SharedErrorStore) {
    let walk = WalkBuilder::new(base.join("facts"))
                           .types(TypesBuilder::new()
                                               .add_defaults()
                                               .select("yaml")
                                               .build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let docs = docs.clone();
        let errors = errors.clone();
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = Path::new(path.path());
                let mut context = ConstructContext::new(path.clone(),
                                                        docs.clone(),
                                                        errors.clone());
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
              errors: SharedErrorStore) {
    let mut types = TypesBuilder::new();
    types.add("osm", "*.osm").unwrap();
    let walk = WalkBuilder::new(base.join("paths"))
                           .types(types.select("osm").build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let docs = docs.clone();
        let errors = errors.clone();
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = Path::new(path.path());
                let mut context = ConstructContext::new(path.clone(),
                                                        docs.clone(),
                                                        errors.clone());
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
