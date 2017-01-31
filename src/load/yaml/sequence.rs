//! Parsing YAML sequences.

use std::ops;
use yaml_rust::yaml;
use yaml_rust::scanner::Marker;
use ::load::path::Path;
use super::stream::{FromYaml, ValueItem};


//------------ Sequence ------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sequence(Vec<ValueItem>);

impl ops::Deref for Sequence {
    type Target = [ValueItem];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Sequence {
    type Item = ValueItem;
    type IntoIter = ::std::vec::IntoIter<ValueItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


//------------ SequenceBuilder -----------------------------------------------

#[derive(Clone, Debug)]
pub struct SequenceBuilder {
    sequence: Sequence,
    path: Path,
    mark: Marker,
}

impl SequenceBuilder {
    pub fn new(path: Path, mark: Marker) -> Self {
        SequenceBuilder {
            sequence: Sequence(Vec::new()),
            path: path,
            mark: mark,
        }
    }
}

impl yaml::Sequence for SequenceBuilder {
    type Item = ValueItem;

    fn push(&mut self, item: Self::Item) {
        self.sequence.0.push(item)
    }

    fn finalize(self) -> Self::Item {
        ValueItem::new(self.sequence.into(), self.path,
                       Some(self.mark))
    }
}

