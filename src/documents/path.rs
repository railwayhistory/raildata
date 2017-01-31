use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::document::DocumentType;


//------------ Path ----------------------------------------------------------

pub struct Path {
    pub key: String,
    /*
    pub nodes: Vec<Node>,
    pub name: String
    */
}

impl Path {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder) -> Result<Path, ()> {
        Ok(Path {
            key: key
        })
    }
}

impl Path {
    pub fn key(&self) -> &str {
        &self.key
    }
}

/*
pub struct Node {
    pub lon: f64,
    pub lat: f64,
    pub pre: f64,
    pub port: f64,
    pub name: Key,
    pub point: Vec<Reference<Point>>,
    pub description: String
}
*/


//------------ PathRef -------------------------------------------------------

pub struct PathRef(DocumentRef);

impl PathRef {
    pub fn get(&self) -> DocumentGuard<Path> {
        self.0.get()
    }
}

impl FromYaml for PathRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(PathRef(builder.ref_doc(item.value(), item.source(),
                                   DocumentType::Path)))
    }
}

