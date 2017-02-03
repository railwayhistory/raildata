//! Loading the facts subdirectory of the working space.

use std::io;
use std::fs::{DirEntry, read_dir};
use std::path::PathBuf;
use ::collection::CollectionBuilder;
use super::path::Path;
use super::yaml::{Stream, Vars};

pub fn load_facts_dir(base: Path, builder: &CollectionBuilder,
                      vars: Vars) {
    let vars = load_vars(&base, vars, &builder);
    let _ = load_yamls(&base, &builder, &vars);
    let _ = load_dirs(&base, &builder, &vars);
}


fn load_vars(base: &Path, vars: Vars, builder: &CollectionBuilder)
             -> Vars {
    let path = base.join("vars.yaml");
    Vars::load(path, Some(vars), builder)
}

fn load_yamls(base: &Path, builder: &CollectionBuilder, vars: &Vars)
              -> io::Result<()> {
    walk_dir(base, builder, |entry| {
        if !entry.file_type()?.is_file() {
            return Ok(())
        }
        if let Some(name) = entry.file_name().to_str() {
            if name != "vars.yaml" && name.ends_with(".yaml") {
                load_yaml(entry.path(), builder, vars)
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn load_yaml(path: PathBuf, builder: &CollectionBuilder, vars: &Vars) {
    Stream::load(builder.clone(), path.into(), vars.clone())
}

fn load_dirs(base: &Path, builder: &CollectionBuilder, vars: &Vars)
             -> io::Result<()> {
    walk_dir(base, builder, |entry| {
        if entry.file_type()?.is_dir() {
            load_facts_dir(entry.path().into(), builder, vars.clone())
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

