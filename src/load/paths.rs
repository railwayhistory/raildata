//! Loading the paths subdirectory of the working space.

use std::{io, mem};
use std::collections::HashSet;
use std::fs::{DirEntry, File, read_dir};
use std::path::PathBuf;
use osmxml::elements::{Osm};
use osmxml::read::read_xml;
use ::collection::CollectionBuilder;
use super::path::Path;


//------------ load_paths_dirs -----------------------------------------------

pub fn load_paths_dir(base: Path, builder: &CollectionBuilder) {
    let _ = load_osms(&base, builder);
    let _ = load_dirs(&base, builder);
}

fn load_osms(base: &Path, builder: &CollectionBuilder) -> io::Result<()> {
    walk_dir(base, builder, |entry| {
        if !entry.file_type()?.is_file() {
            return Ok(())
        }
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".osm") {
                load_osm(entry.path(), builder)
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn load_osm(path: PathBuf, builder: &CollectionBuilder) {
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            builder.error((Path::from(path), err));
            return;
        }
    };
    let osm = match read_xml(file) {
        Ok(osm) => osm,
        Err(err) => {
            builder.error((Path::from(path), err));
            return;
        }
    };
    build_path_docs(&path.into(), osm, builder);
}

fn load_dirs(base: &Path, builder: &CollectionBuilder) -> io::Result<()>  {
    walk_dir(base, builder, |entry| {
        if entry.file_type()?.is_dir() {
            load_paths_dir(entry.path().into(), builder)
        }
        Ok(())
    })?;
    Ok(())
}


fn walk_dir<F>(base: &Path, builder: &CollectionBuilder, mut op: F)
               -> io::Result<()>
            where F: FnMut(DirEntry) -> io::Result<()> {
    let dir = match read_dir(base) {
        Ok(dir) => dir,
        Err(err) => {
            builder.error((base.clone(),
                           format!("cannot read directory: {}", &err)));
            return Err(err);
        }
    };
    for entry in dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                builder.error((base.clone(),
                               format!("Cannot read directory: {}", &err)));
                return Err(err);
            }
        };
        op(entry)?;
    }
    Ok(())
}


//------------ build_path_docs -----------------------------------------------

fn build_path_docs(path: &Path, mut osm: Osm, builder: &CollectionBuilder) {
    // Swap out the relations so we don’t hold a mutable reference to
    // `osm` while draining the relations.
    let mut relations = HashSet::new();
    mem::swap(osm.relations_mut(), &mut relations);
    for rel in relations.drain() {
        match ::documents::path::Path::from_osm(rel, &osm, path, builder) {
            Ok(doc) => {
                if let Err((doc, org)) = builder.update_doc(doc,
                                                            path.clone().into()) {
                    builder.error((path.clone(),
                        format!("duplicate document '{}'. First defined at {}.",
                                doc.key(), org)))
                }
            }
            Err(Some(key)) => builder.broken_doc(key, path.clone().into()),
            Err(None) => { }
        }
    }
}

