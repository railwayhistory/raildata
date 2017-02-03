//! Parsing YAML mappings.

use std::ops;
use std::collections::btree_map::{BTreeMap, IntoIter};
use yaml_rust::yaml;
use yaml_rust::scanner::Marker;
use ::collection::CollectionBuilder;
use ::load::path::Path;
use super::stream::{FromYaml, ValueItem};


//------------ Mapping -------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mapping(BTreeMap<String, ValueItem>);

impl Mapping {
    pub fn new() -> Self {
        Mapping(BTreeMap::new())
    }

    pub fn parse_opt<T: FromYaml>(&mut self, key: &str,
                                  builder: &CollectionBuilder)
                                  -> Result<Option<T>, ()> {
        match self.0.remove(key) {
            None => Ok(None),
            Some(item) => {
                if item.is_null() {
                    Ok(None)
                }
                else {
                    T::from_yaml(item, builder).map(Some)
                }
            }
        }
    }

    pub fn parse_default<T>(&mut self, key: &str,
                            builder: &CollectionBuilder)
                            -> Result<T, ()>
                         where T: FromYaml + Default {
        match self.0.remove(key) {
            None => Ok(T::default()),
            Some(item) => T::from_yaml(item, builder)
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
    builder: CollectionBuilder,
}

impl MappingBuilder {
    pub fn new(path: Path, mark: Marker, builder: CollectionBuilder)
               -> Self {
        MappingBuilder {
            mapping: Mapping(BTreeMap::new()),
            path: path,
            mark: mark,
            builder: builder,
        }
    }
}

impl yaml::Mapping for MappingBuilder {
    type Item = ValueItem;

    fn insert(&mut self, key: Self::Item, value: Self::Item) {
        let key = match key.into_string(&self.builder) {
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


