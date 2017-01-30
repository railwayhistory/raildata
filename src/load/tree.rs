
use std::sync::{Arc, Mutex};
use ::collection::{Collection, CollectionBuilder};
use super::error::{Error, ErrorGatherer};
use super::facts::load_facts_dir;
use super::path::Path;
use super::yaml::Vars;


pub fn load_tree(path: Path) -> Result<Collection, Vec<Error>> {
    let mut tree = Tree::new(path);
    tree.load();
    tree.finalize()
}


pub struct Tree {
    base: Path,
    collection: Arc<Mutex<CollectionBuilder>>,
    errors: ErrorGatherer,
}

impl Tree {
    pub fn new(path: Path) -> Self {
        Tree {
            base: path,
            collection: Arc::new(Mutex::new(CollectionBuilder::new())),
            errors: ErrorGatherer::new(),
        }
    }

    pub fn load(&mut self) {
        load_facts_dir(self.base.clone(), self.collection.clone(),
                       Vars::new(None), self.errors.clone());
    }

    pub fn finalize(self) -> Result<Collection, Vec<Error>> {
        let res = Arc::try_unwrap(self.collection).unwrap()
                      .into_inner().unwrap().finalize(&self.errors);
        match res {
            None => Err(self.errors.unwrap()),
            Some(coll) => {
                if self.errors.is_empty() {
                    Ok(coll)
                }
                else {
                    Err(self.errors.unwrap())
                }
            }
        }
    }
}
