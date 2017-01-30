
use std::{cmp, fmt, hash, ops};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use url::Url;
use yaml_rust::yaml;
use yaml_rust::scanner::{Marker, TScalarStyle, TokenType};
use ::collection::CollectionBuilder;
use ::documents::Document;
use ::load::{Error, ErrorGatherer, Source};
use ::load::path::Path;
use super::mapping::{Mapping, MappingBuilder};
use super::sequence::{Sequence, SequenceBuilder};
use super::scalar::{Scalar, parse_scalar};
use super::vars::Vars;


//------------ Stream --------------------------------------------------------

pub struct Stream {
    path: Path,
    collection: Arc<Mutex<CollectionBuilder>>,
    vars: Vars,
    errors: ErrorGatherer,
}

impl Stream {
    pub fn load(collection: Arc<Mutex<CollectionBuilder>>, path: Path,
                vars: Vars, errors: ErrorGatherer) {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                errors.add(Error::file(path, err));
                return;
            }
        };
        let mut data = Vec::new();
        if let Err(err) = file.read_to_end(&mut data) {
            errors.add(Error::file(path, format!("Reading failed: {}", err)));
            return;
        }
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => {
                errors.add(Error::file(path, format!("Not UTF-8 data.")));
                return;
            }
        };
        let mut stream = Stream::new(collection, path.clone(), vars,
                                     errors.clone());
        if let Err(err) = yaml::GenericYamlLoader::load_from_str(&data,
                                                                 &mut stream) {
            errors.add(Error::file(path, err))
        }
    }
}

impl Stream {
    pub fn new(collection: Arc<Mutex<CollectionBuilder>>, path: Path,
               vars: Vars, errors: ErrorGatherer)
               -> Self {
        Stream {
            collection: collection,
            path: path,
            vars: vars,
            errors: errors,
        }
    }

    pub fn path(&self) -> Path {
        self.path.clone()
    }

    pub fn source(&self, mark: Option<Marker>) -> Source {
        match mark {
            Some(mark) => Source::in_file(self.path.clone(), mark),
            None => Source::file(self.path.clone())
        }
    }

    pub fn get_var(&self, var: &str) -> Option<ValueItem> {
        self.vars.get(var)
    }

    pub fn push_error(&self, error: Error) {
        self.errors.add(error)
    }
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stream")
         .field("path", &self.path)
         .finish()
    }
}

impl yaml::Stream for Stream {
    type Item = ValueItem;
    type Sequence = SequenceBuilder;
    type Mapping = MappingBuilder;

    fn create_scalar(&self, value: &str, _style: TScalarStyle,
                     tag: &Option<TokenType>, mark: Marker) -> Self::Item {
        match parse_scalar(value, tag, &self.path, mark, &self.vars) {
            Ok(res) => res,
            Err(err) => {
                let pos = self.source(Some(mark));
                self.push_error(Error::new(pos, err));
                ValueItem::new(Value::Empty, self.path(), Some(mark))
            }
        }
    }

    fn create_bad_value(&self) -> Self::Item {
        Item::new(Value::Empty, self.path(), None)
    }

    fn create_sequence(&self, mark: Marker) -> Self::Sequence {
        SequenceBuilder::new(self.path.clone(), mark)
    }

    fn create_mapping(&self, mark: Marker) -> Self::Mapping {
        MappingBuilder::new(self.path.clone(), mark, self.errors.clone())
    }

    fn create_document(&mut self, item: Self::Item) {
        let pos = item.source(); 
        if let Ok(doc) = Document::from_yaml(item,
                                             &mut self.collection.lock()
                                                                 .unwrap(),
                                             &self.errors) {
            if let Err((doc, org)) = self.collection.lock().unwrap()
                                                    .update_doc(doc,
                                                                pos.clone()) {
                self.push_error(Error::new(pos,
                    format!("duplicate document '{}'. First defined at {}.",
                            doc.key(), org)))
            }
        }
    }
}


//------------ Value ---------------------------------------------------------

/// A YAML value.
///
/// Represents the data of something inside a YAML document.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Scalar(Scalar),
    String(String),
    Sequence(Sequence),
    Mapping(Mapping),
    Empty,
}

impl Value {
    pub fn into_mapping(self) -> Option<Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }
}

impl From<Mapping> for Value {
    fn from(m: Mapping) -> Self {
        Value::Mapping(m)
    }
}

impl From<Sequence> for Value {
    fn from(s: Sequence) -> Self {
        Value::Sequence(s)
    }
}


//------------ Item ----------------------------------------------------------

/// A YAML item as some value plus source information.
///
/// This has a lifetime because it references the stream that contains it.
#[derive(Clone, Debug)]
pub struct Item<V> {
    value: V,
    path: Path,
    mark: Option<Marker>,
}

pub type ValueItem = Item<Value>;


impl<V> Item<V> {
    pub fn new(value: V, path: Path, mark: Option<Marker>) -> Self {
        Item {
            value: value,
            path: path,
            mark: mark,
        }
    }

    pub fn into_inner(self) -> (V, Source) {
        let source = self.source();
        (self.value, source)
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn source(&self) -> Source {
        match self.mark {
            Some(mark) => Source::in_file(self.path.clone(), mark),
            None => Source::file(self.path.clone()),
        }
    }
}

impl Item<Value> {
    pub fn into_string_item(self, errors: &ErrorGatherer)
                            -> Result<Item<String>, ()> {
        match self.value {
            Value::String(s) => Ok(Item::new(s, self.path, self.mark)),
            _ => {
                errors.add(Error::new(self.into(),
                                      String::from("expected string")));
                Err(())
            }
        }
    }

    pub fn into_string(self, errors: &ErrorGatherer) -> Result<String, ()> {
        match self.value {
            Value::String(s) => Ok(s),
            _ => {
                errors.add(Error::new(self.into(),
                                      String::from("expected string")));
                Err(())
            }
        }
    }

    pub fn into_mapping(self, errors: &ErrorGatherer)
                        -> Result<Item<Mapping>, ()> {
        match self.value {
            Value::Mapping(m) => Ok(Item::new(m, self.path, self.mark)),
            _ => {
                errors.add((self.into(), String::from("expected mapping")));
                Err(())
            }
        }
    }

    pub fn try_into_sequence(self) -> Result<Sequence, Self> {
        match self.value {
            Value::Sequence(s) => Ok(s),
            _ => Err(self)
        }
    }

    pub fn into_sequence_item(self, errors: &ErrorGatherer)
                         -> Result<Item<Sequence>, ()> {
        match self.value {
            Value::Sequence(s) => Ok(Item::new(s, self.path, self.mark)),
            _ => {
                errors.add((self.into(), String::from("expected sequence")));
                Err(())
            }
        }
    }

    pub fn into_sequence(self, errors: &ErrorGatherer)
                         -> Result<Sequence, ()> {
        match self.value {
            Value::Sequence(s) => Ok(s),
            _ => {
                errors.add((self.into(), String::from("expected sequence")));
                Err(())
            }
        }
    }
}

impl Item<Mapping> {
    pub fn parse<T: FromYaml>(&mut self, key: &str,
                              collection: &mut CollectionBuilder,
                              errors: &ErrorGatherer)
                              -> Result<T, ()> {
        match self.value.remove(key) {
            None => {
                errors.add((self.source(),
                            format!("missing key '{}'", key)));
                Err(())
            }
            Some(item) => {
                T::from_yaml(item, collection, errors)
            }
        }
    }


    pub fn mandatory_key(&mut self, key: &str, errors: &ErrorGatherer)
                         -> Result<ValueItem, ()> {
        if let Some(item) = self.value.remove(key) {
            Ok(item)
        }
        else {
            errors.add(Error::new(self.source(),
                                  format!("missing key '{}'", key)));
            Err(())
        }
    }

    pub fn optional_key(&mut self, key: &str) -> Option<ValueItem> {
        self.value.remove(key)
    }
}


//--- From

impl<V> From<Item<V>> for Source {
    fn from(item: Item<V>) -> Source {
        item.source()
    }
}


//--- Deref, DerefMut, AsRef

impl<V> ops::Deref for Item<V> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.value
    }
}

impl<V> ops::DerefMut for Item<V> {
    fn deref_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<V> AsRef<V> for Item<V> {
    fn as_ref(&self) -> &V {
        &self.value
    }
}


//--- Eq, Hash, Ord, PartialEq, PartialOrd

impl<V: Eq> Eq for Item<V> { }

impl<V: hash::Hash> hash::Hash for Item<V> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl<V: Ord> Ord for Item<V> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<V: PartialEq> PartialEq for Item<V> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<V: PartialOrd> PartialOrd for Item<V> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}


//------------ FromYaml ------------------------------------------------------

pub trait FromYaml: Sized {
    fn from_yaml(item: ValueItem, collection: &mut CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<Self, ()>;
}

impl FromYaml for String {
    fn from_yaml(item: ValueItem, _: &mut CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<Self, ()> {
        item.into_string(errors)
    }
}

impl FromYaml for Url {
    fn from_yaml(item: ValueItem, _: &mut CollectionBuilder,
                 errors: &ErrorGatherer) -> Result<Self, ()> {
        let item = item.into_string_item(errors)?;
        Url::parse(&item.value()).map_err(|err| {
            errors.add((item.source(),
                        format!("illegal URL: {}", err)));
        })
    }
}
 
