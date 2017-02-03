
use std::{cmp, fmt, hash, ops};
use std::fs::File;
use std::io::Read;
use url::Url;
use yaml_rust::yaml;
use yaml_rust::scanner::{Marker, TScalarStyle, TokenType};
use ::collection::{CollectionBuilder, DocumentRef};
use ::documents::Document;
use ::load::{Error, Source};
use ::load::path::Path;
use super::mapping::{Mapping, MappingBuilder};
use super::sequence::{Sequence, SequenceBuilder};
use super::scalar::{Float, parse_scalar};
use super::vars::Vars;


//------------ Stream --------------------------------------------------------

pub struct Stream {
    path: Path,
    builder: CollectionBuilder,
    vars: Vars,
}

impl Stream {
    pub fn load(builder: CollectionBuilder, path: Path,
                vars: Vars) {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                builder.error((path, err));
                return;
            }
        };
        let mut data = Vec::new();
        if let Err(err) = file.read_to_end(&mut data) {
            builder.error((path, format!("Reading failed: {}", err)));
            return;
        }
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => {
                builder.error((path, format!("Not UTF-8 data.")));
                return;
            }
        };
        let mut stream = Stream::new(builder, path.clone(), vars);
        if let Err(err) = yaml::GenericYamlLoader::load_from_str(&data,
                                                                 &mut stream) {
            stream.error((path, err))
        }
    }
}

impl Stream {
    pub fn new(builder: CollectionBuilder, path: Path,
               vars: Vars) -> Self {
        Stream {
            builder: builder,
            path: path,
            vars: vars,
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

    pub fn error<E: Into<Error>>(&self, err: E) {
        self.builder.error(err)
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
                self.builder.error((pos, err));
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
        MappingBuilder::new(self.path.clone(), mark,
                            self.builder.clone())
    }

    fn create_document(&mut self, item: Self::Item) {
        let pos = item.source(); 
        match Document::from_yaml(item, &self.builder) {
            Ok(doc) => {
                if let Err((doc, org)) = self.builder.update_doc(doc,
                                                                 pos.clone()) {
                    self.builder.error((pos,
                        format!("duplicate document '{}'. First defined at {}.",
                                doc.key(), org)))
                }
            }
            Err(Some(key)) => self.builder.broken_doc(key, pos),
            Err(None) => { }
        }
    }
}


//------------ Value ---------------------------------------------------------

/// A YAML value.
///
/// Represents the data of something inside a YAML document.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Sequence(Sequence),
    Mapping(Mapping),
    String(String),
    Null,
    Bool(bool),
    Int(i64),
    Float(Float),
    Empty,
}

impl Value {
    pub fn into_mapping(self) -> Option<Mapping> {
        match self {
            Value::Mapping(m) => Some(m),
            _ => None,
        }
    }

    pub fn style(&self) -> &'static str {
        match *self {
            Value::Sequence(_) => "sequence",
            Value::Mapping(_) => "mapping",
            Value::String(_) => "string",
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "integer",
            Value::Float(_) => "float",
            Value::Empty => "empty",
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
    pub fn parse<T: FromYaml>(self, builder: &CollectionBuilder)
                              -> Result<T, ()> {
        T::from_yaml(self, builder)
    }

    pub fn is_null(&self) -> bool {
        if let Value::Null = self.value { true }
        else { false }
    }

    pub fn into_string_item(self, builder: &CollectionBuilder)
                            -> Result<Item<String>, ()> {
        match self.value {
            Value::String(s) => Ok(Item::new(s, self.path, self.mark)),
            Value::Int(i) => Ok(Item::new(format!("{}", i), self.path,
                                          self.mark)),
            _ => {
                builder.error((self.source(),
                               format!("expected string, got {}",
                                       self.value.style())));
                Err(())
            }
        }
    }

    pub fn into_string(self, builder: &CollectionBuilder)
                       -> Result<String, ()> {
        match self.value {
            Value::String(s) => Ok(s),
            Value::Int(i) => Ok(format!("{}", i)),
            _ => {
                builder.error((self.source(),
                               format!("expected string, got {}",
                                       self.value.style())));
                Err(())
            }
        }
    }

    pub fn into_mapping(self, builder: &CollectionBuilder)
                        -> Result<Item<Mapping>, ()> {
        match self.value {
            Value::Mapping(m) => Ok(Item::new(m, self.path, self.mark)),
            _ => {
                builder.error((self.source(),
                               String::from("expected mapping")));
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

    pub fn into_sequence_item(self, builder: &CollectionBuilder)
                         -> Result<Item<Sequence>, ()> {
        match self.value {
            Value::Sequence(s) => Ok(Item::new(s, self.path, self.mark)),
            _ => {
                builder.error((self.source(),
                               String::from("expected sequence")));
                Err(())
            }
        }
    }

    pub fn into_sequence(self, builder: &CollectionBuilder)
                         -> Result<Sequence, ()> {
        match self.value {
            Value::Sequence(s) => Ok(s),
            _ => {
                builder.error((self.source(),
                               String::from("expected sequence")));
                Err(())
            }
        }
    }

    pub fn into_float(self, builder: &CollectionBuilder)
                      -> Result<Float, ()> {
        match self.value {
            Value::Float(f) => Ok(f),
            _ => {
                builder.str_error(self.source(), "expected float");
                Err(())
            }
        }
    }

    pub fn into_int_item(self, builder: &CollectionBuilder)
                         -> Result<Item<i64>, ()> {
        match self.value {
            Value::Int(i) => Ok(Item::new(i, self.path, self.mark)),
            _ => {
                builder.str_error(self.source(), "expected integer");
                Err(())
            }
        }
    }

    pub fn into_int(self, builder: &CollectionBuilder)
                    -> Result<i64, ()> {
        match self.value {
            Value::Int(i) => Ok(i),
            _ => {
                builder.str_error(self.source(), "expected integer");
                Err(())
            }
        }
    }
}

impl Item<Mapping> {
    pub fn parse_mandatory<T: FromYaml>(&mut self, key: &str,
                                        builder: &CollectionBuilder)
                                        -> Result<T, ()> {
        match self.value.remove(key) {
            None => {
                builder.error((self.source(),
                               format!("missing key '{}'", key)));
                Err(())
            }
            Some(item) => {
                T::from_yaml(item, builder)
            }
        }
    }

    pub fn mandatory_key(&mut self, key: &str, builder: &CollectionBuilder)
                         -> Result<ValueItem, ()> {
        if let Some(item) = self.value.remove(key) {
            Ok(item)
        }
        else {
            builder.error((self.source(), format!("missing key '{}'", key)));
            Err(())
        }
    }

    pub fn optional_key(&mut self, key: &str) -> Option<ValueItem> {
        self.value.remove(key)
    }

    pub fn exhausted(&self, builder: &CollectionBuilder) -> Result<(), ()> {
        if self.value.is_empty() {
            Ok(())
        }
        else {
            let mut s = String::from("extra elements in mapping: ");
            let mut first = true;
            for key in self.value.keys() {
                if !first {
                    s.push_str(", ");
                }
                s.push_str(&key);
                first = false;
            }
            builder.error((self.source(), s));
            Err(())
        }
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
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()>;
}

impl<T: FromYaml> FromYaml for Vec<T> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_sequence(builder)?;
        let mut res = Some(Vec::new());
        for t in item {
            if let Ok(t) = T::from_yaml(t, builder) {
                if let Some(ref mut res) = res {
                    res.push(t)
                }
            }
            else {
                res = None
            }
        }
        match res {
            Some(res) => Ok(res),
            None => Err(())
        }
    }
}

impl FromYaml for String {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_string(builder)
    }
}

impl FromYaml for Item<String> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_string_item(builder)
    }
}

impl FromYaml for Option<String> {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let (value, pos) = item.into_inner();
        match value {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            _ => {
                builder.str_error(pos, "expected string or null");
                Err(())
            }
        }
    }
}

impl FromYaml for Url {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Url::parse(&item.value()).map_err(|err| {
            builder.error((item.source(),
                           format!("illegal URL: {}", err)));
        })
    }
}

impl FromYaml for bool {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        match *item.value() {
            Value::Bool(v) => Ok(v),
            _ => {
                builder.str_error(item.source(), "expected boolean");
                Err(())
            }
        }
    }
}
 
impl FromYaml for f64 {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_float(builder).map(|f| f.to_f64())
    }
}

impl FromYaml for i64 {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_int(builder)
    }
}

impl FromYaml for u32 {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_int_item(builder).and_then(|item| {;
            let (v, source) = item.into_inner();
            if v >= 0 && v <= ::std::u32::MAX as i64 {
                Ok(v as u32)
            }
            else {
                builder.str_error(source, "value out of range");
                Err(())
            }
        })
    }
}

impl FromYaml for u16 {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_int_item(builder).and_then(|item| {
            let (v, source) = item.into_inner();
            if v >= 0 && v <= ::std::u16::MAX as i64 {
                Ok(v as u16)
            }
            else {
                builder.str_error(source, "value out of range");
                Err(())
            }
        })
    }
}

impl FromYaml for u8 {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        item.into_int_item(builder).and_then(|item| {
            let (v, source) = item.into_inner();
            if v >= 0 && v <= ::std::u8::MAX as i64 {
                Ok(v as u8)
            }
            else {
                builder.str_error(source, "value out of range");
                Err(())
            }
        })
    }
}


impl FromYaml for DocumentRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(builder.ref_doc(item.value(), item.source(), None))
    }
}

