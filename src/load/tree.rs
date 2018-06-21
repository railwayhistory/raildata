
use std::{io, mem, path};
use std::collections::HashSet;
use std::fs::File;
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use osmxml::read::read_xml;
use ::document::path::Path;
use ::document::store::{DocumentStore, DocumentStoreBuilder};
use ::types::{IntoMarked, Location};
use super::read::Utf8Chars;
use super::report::{self, PathReporter, Report, Reporter, Stage};
use super::yaml::Loader;


//------------ load_tree -----------------------------------------------------

pub fn load_tree(path: &path::Path) -> Result<DocumentStore, Report> {
    let builder = DocumentStoreBuilder::new();
    let report = Reporter::new();

    // Phase 1: Construct all documents and check that they are all present
    //          and accounted for.
    load_facts(path, builder.clone(), report.clone());
    load_paths(path, builder.clone(), report.clone());

    let res = {
        // Separate block so the report clone is dropped before we unwrap it
        // later.
        builder.into_store(&mut report.clone().stage(Stage::Translate))
    };
    match res {
        Ok(some) => Ok(some),
        Err(_) => {
            Err(report.unwrap())
        }
    }
}


//------------ load_facts ----------------------------------------------------

fn load_facts(
    base: &path::Path,
    docs: DocumentStoreBuilder,
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
        let mut docs = docs.clone();
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

fn load_paths(
    base: &path::Path,
    docs: DocumentStoreBuilder,
    report: Reporter
) {
    let mut types = TypesBuilder::new();
    types.add("osm", "*.osm").unwrap();
    let walk = WalkBuilder::new(base.join("paths"))
                           .types(types.select("osm").build().unwrap())
                           .build_parallel();
    walk.run(|| {
        let mut docs = docs.clone();
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
                    Ok(mut file) => {
                        let mut report = report.clone()
                            .stage(Stage::Translate)
                            .with_path(path);
                        load_osm_file(&mut file, &mut docs, &mut report);
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

pub fn load_osm_file<R: io::Read>(
    read: &mut R,
    docs: &mut DocumentStoreBuilder,
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
        match Path::from_osm(relation, &osm, docs, report) {
            Ok(path) => {
                if let Err(err) = docs.insert(path.key().clone(), path,
                                                    Location::NONE, report) {
                    report.error(err.marked(Location::NONE))
                }
            }
            Err(Some(key)) => {
                if let Err(err) = docs.insert_broken::<Path>(key,
                                                    Location::NONE, report) {
                    report.error(err.marked(Location::NONE))
                }
            }
            Err(None) => { }
        }
    }
}

