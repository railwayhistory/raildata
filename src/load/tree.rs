
use std::{io, mem, path};
use std::collections::HashSet;
use std::fs::File;
use ignore::{WalkBuilder, WalkState};
use ignore::types::TypesBuilder;
use osmxml::read::read_xml;
use crate::document::Path;
use crate::document::common::DocumentType;
use crate::library::{LibraryBuilder, Library};
use crate::types::{IntoMarked, Location};
use super::read::Utf8Chars;
use super::report::{self, PathReporter, Report, Reporter, Stage};
use super::yaml::Loader;


//------------ load_tree -----------------------------------------------------

pub fn load_tree(path: &path::Path) -> Result<Library, Report> {
    let report = Reporter::new();

    // Phase 1: Construct all documents and check that they are all present
    //          and accounted for.
    let store = {
        let builder = LibraryBuilder::new();
        load_facts(path, builder.clone(), report.clone());
        load_paths(path, builder.clone(), report.clone());
        builder.into_library_mut(&mut report.clone().stage(Stage::Translate))
    };
    let store = match store {
        Ok(store) => store,
        Err(_) => return Err(report.unwrap())
    };

    // Phase 2:  Verify

    // Phase 3: Profit
    Ok(store.into_library())
}


//------------ load_facts ----------------------------------------------------

fn load_facts(
    base: &path::Path,
    docs: LibraryBuilder,
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
    base: &path::Path,
    docs: LibraryBuilder,
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
                    Ok(mut file) => {
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
    docs: &LibraryBuilder,
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
                let _ = docs.insert(
                    path.key().clone(), path.into(), report
                );
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

