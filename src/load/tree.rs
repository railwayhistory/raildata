
use std::{io, mem};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use osmxml::read::read_xml;
use crate::document::path;
use crate::document::common::DocumentType;
use crate::store::{DataStore, StoreLoader};
use crate::types::{IntoMarked, Location};
use super::read::Utf8Chars;
use super::report::{self, PathReporter, Report, Reporter, Stage};
use super::yaml::Loader;


//------------ load_tree -----------------------------------------------------

pub fn load_tree(path: &Path) -> Result<DataStore, Report> {
    let report = Reporter::new();

    let store = {
        let builder = Arc::new(StoreLoader::new());
        load_facts(path, builder.clone(), report.clone());
        load_paths(path, builder.clone(), report.clone());
        let builder = Arc::try_unwrap(builder).unwrap();
        builder.into_data_store(&mut report.clone().stage(Stage::Translate))
    };
    let store = match store {
        Ok(store) => store,
        Err(_) => return Err(report.unwrap())
    };
    if !report.is_empty() {
        return Err(report.unwrap())
    }
    Ok(store)
}


//------------ load_facts ----------------------------------------------------

fn load_facts(
    base: &Path,
    docs: Arc<StoreLoader>,
    report: Reporter
) {
    let walk = WalkBuilder::new(base.join("facts"))
        .types(TypesBuilder::new()
            .add_defaults()
            .select("yaml")
            .build().unwrap()
        )
        .build_parallel();
    walk.run(|| {
        let docs = docs.clone();
        let report = report.clone();
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = report::Path::new(path.path());
                match File::open(&path) {
                    Ok(file) => {
                        let file = BufReader::new(file);
                        let mut report = report.clone()
                            .stage(Stage::Translate)
                            .with_path(path);
                        let res = {
                            let mut loader = Loader::new(|v| {
                                let _ = docs.from_yaml(v, &mut report);
                            });
                            loader.load(Utf8Chars::new(file))
                        };
                        if let Err(err) = res {
                            let mut report = report.restage(Stage::Parse);
                            report.error(err.marked(Location::NONE));
                        }
                    }
                    Err(err) => {
                        report.clone().stage(Stage::Parse)
                            .with_path(path).error(err.marked(Location::NONE))
                    }
                }
            }
            WalkState::Continue
        })
    })
}


//------------ load_paths ----------------------------------------------------

pub fn load_paths(
    base: &Path,
    docs: Arc<StoreLoader>,
    report: Reporter
) {
    let mut types = TypesBuilder::new();
    types.add("osm", "*.osm").unwrap();
    let walk = WalkBuilder::new(base.join("paths"))
                           .types(types.select("osm").build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let docs = docs.clone();
        let report = report.clone();
        Box::new(move |path| {
            if let Ok(path) = path {
                if let Some(file_type) = path.file_type() {
                    if file_type.is_dir() {
                        return WalkState::Continue
                    }
                }
                let path = report::Path::new(path.path());
                match File::open(&path) {
                    Ok(file) => {
                        let mut file = BufReader::new(file);
                        let mut report = report.clone()
                            .stage(Stage::Translate)
                            .with_path(path);
                        load_osm_file(&mut file, &docs, &mut report);
                    }
                    Err(err) => {
                        report.clone().stage(Stage::Parse)
                            .with_path(path).error(err.marked(Location::NONE))
                    }
                }
            }
            WalkState::Continue
        })
    })
}


//------------ load_osm_file -------------------------------------------------

fn load_osm_file<R: io::Read>(
    read: &mut R,
    docs: &StoreLoader,
    report: &mut PathReporter
) {
    let mut osm = match read_xml(read) {
        Ok(osm) => osm,
        Err(err) => {
            report.error(err.unmarked());
            return;
        }
    };
    
    // Swap out the relations so we don’t hold a mutable reference to
    // `osm` while draining the relations.
    let mut relations = HashSet::new();
    mem::swap(osm.relations_mut(), &mut relations);
    for relation in relations.drain() {
        match path::Data::from_osm(relation, &osm, docs, report) {
            Ok(path) => {
                let _ = docs.insert(path.into(), report);
            }
            Err(Some(key)) => {
                let _ = docs.insert_broken(
                    key, Some(DocumentType::Path), Location::NONE, report
                );
            }
            Err(None) => { }
        }
    }
}

