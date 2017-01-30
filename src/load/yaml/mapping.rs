//! Parsing YAML mappings.

use std::ops;
use std::collections::btree_map::{BTreeMap, IntoIter};
use yaml_rust::yaml;
use yaml_rust::scanner::Marker;
use ::collection::CollectionBuilder;
use ::load::path::Path;
use ::load::error::ErrorGatherer;
use super::stream::{FromYaml, ValueItem};


//------------ Mapping -------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mapping(BTreeMap<String, ValueItem>);

impl Mapping {
    pub fn new() -> Self {
        Mapping(BTreeMap::new())
    }

    pub fn parse_opt<T: FromYaml>(&mut self, key: &str,
                                  collection: &mut CollectionBuilder,
                                  errors: &ErrorGatherer)
                                  -> Result<Option<T>, ()> {
        match self.0.remove(key) {
            None => Ok(None),
            Some(item) => Ok(Some(T::from_yaml(item, collection, errors)?)),
        }
    }
}

impl IntoIterator for Mapping {
    type Item = (String, ValueItem);
    type IntoIter = IntoIter<String, ValueItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ops::Deref for Mapping {
    type Target = BTreeMap<String, ValueItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Mapping {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


//------------ MappingBuilder ------------------------------------------------

#[derive(Clone, Debug)]
pub struct MappingBuilder {
    mapping: Mapping,
    path: Path,
    mark: Marker,
    errors: ErrorGatherer,
}

impl MappingBuilder {
    pub fn new(path: Path, mark: Marker, errors: ErrorGatherer)
               -> Self {
        MappingBuilder {
            mapping: Mapping(BTreeMap::new()),
            path: path,
            mark: mark,
            errors: errors,
        }
    }
}

impl yaml::Mapping for MappingBuilder {
    type Item = ValueItem;

    fn insert(&mut self, key: Self::Item, value: Self::Item) {
        let key = match key.into_string(&self.errors) {
            Ok(item) => item,
            Err(_) => return
        };
        self.mapping.0.insert(key, value);
    }

    fn finalize(self) -> Self::Item {
        ValueItem::new(self.mapping.into(), self.path,
                       Some(self.mark))
    }
}



