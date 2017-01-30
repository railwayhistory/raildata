//! Loading the facts subdirectory of the working space.

use std::io;
use std::fs::{DirEntry, read_dir};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use ::collection::CollectionBuilder;
use super::error::{Error, ErrorGatherer};
use super::path::Path;
use super::yaml::{Stream, Vars};

pub fn load_facts_dir(base: Path,
                      collection: Arc<Mutex<CollectionBuilder>>,
                      vars: Vars,
                      errors: ErrorGatherer) {
    let vars = load_vars(&base, vars, &errors);
    let _ = load_yamls(&base, &collection, &vars, &errors);
    let _ = load_dirs(&base, &collection, &vars, &errors);
}


fn load_vars(base: &Path, vars: Vars, errors: &ErrorGatherer)
             -> Vars {
    let path = base.join("vars.yaml");
    Vars::load(path, Some(vars), errors.clone())
}

fn load_yamls(base: &Path, 
              collection: &Arc<Mutex<CollectionBuilder>>,
              vars: &Vars, errors: &ErrorGatherer)
              -> io::Result<()> {
    walk_dir(base, errors, |entry, errors| {
        if !entry.file_type()?.is_file() {
            return Ok(())
        }
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".yaml") {
                load_yaml(entry.path(), collection, vars, errors)
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn load_yaml(path: PathBuf,
             collection: &Arc<Mutex<CollectionBuilder>>,
             vars: &Vars, errors: &ErrorGatherer) {
    Stream::load(collection.clone(), path.into(), vars.clone(), errors.clone())
}

fn load_dirs(base: &Path, 
             collection: &Arc<Mutex<CollectionBuilder>>,
             vars: &Vars, errors: &ErrorGatherer)
             -> io::Result<()> {
    walk_dir(base, errors, |entry, errors| {
        if entry.file_type()?.is_dir() {
            load_facts_dir(entry.path().into(), collection.clone(),
                           vars.clone(), errors.clone())
        }
        Ok(())
    })?;
    Ok(())
}


fn walk_dir<F>(base: &Path, errors: &ErrorGatherer, mut op: F)
               -> io::Result<()>
            where F: FnMut(DirEntry, &ErrorGatherer) -> io::Result<()> {
    let dir = match read_dir(base) {
        Ok(dir) => dir,
        Err(err) => {
            errors.add(Error::global(format!("Cannot read directory {}: {}",
                                              base.display(), &err)));
            return Err(err);
        }
    };
    for entry in dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                errors.add(Error::global(
                        format!("Cannot read directory {}: {}",
                        base.display(), &err)));
                return Err(err);
            }
        };
        op(entry, errors)?;
    }
    Ok(())
}

