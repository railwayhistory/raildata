use ::collection::{Collection, CollectionBuilder};
use super::error::Error;
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
    collection: CollectionBuilder,
}

impl Tree {
    pub fn new(path: Path) -> Self {
        Tree {
            base: path,
            collection: CollectionBuilder::new(),
        }
    }

    pub fn load(&mut self) {
        load_facts_dir(self.base.clone(), &self.collection, Vars::new(None));
    }

    pub fn finalize(self) -> Result<Collection, Vec<Error>> {
        self.collection.finalize().unwrap()
    }
}