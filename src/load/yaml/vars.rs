
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use yaml_rust::yaml;
use yaml_rust::scanner::{Marker, TScalarStyle, TokenType};
use ::collection::CollectionBuilder;
use ::load::error::Source;
use ::load::path::Path;
use super::stream::{Item, Value, ValueItem};
use super::mapping::MappingBuilder;
use super::sequence::SequenceBuilder;
use super::scalar::parse_scalar;


//------------ Vars ----------------------------------------------------------

#[derive(Debug)]
pub struct Vars(Arc<Mutex<VarsInner>>);

impl Vars {
    pub fn new(parent: Option<Vars>) -> Self {
        Vars(Arc::new(Mutex::new(VarsInner::new(parent))))
    }

    pub fn load(path: Path, parent: Option<Vars>, builder: &CollectionBuilder)
                -> Vars {
        let parent = match parent {
            Some(parent) => parent,
            None => Vars::new(None),
        };
        VarsStream::load(path, parent, builder)
    }

    pub fn insert(&mut self, key: String, value: ValueItem) {
        self.0.lock().unwrap().insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<ValueItem> {
        self.0.lock().unwrap().get(key).map(|value| value.clone())
    }
}

impl Clone for Vars {
    fn clone(&self) -> Self {
        Vars(self.0.clone())
    }
}


//------------ VarsInner -----------------------------------------------------

#[derive(Debug)]
struct VarsInner {
    map: Option<HashMap<String, ValueItem>>,
    parent: Option<Vars>,
}

impl VarsInner {
    fn new(parent: Option<Vars>) -> Self {
        VarsInner {
            map: None,
            parent: parent
        }
    }

    fn get(&self, key: &str) -> Option<ValueItem> {
        if let Some(ref map) = self.map {
            if let Some(res) = map.get(key) {
                return Some(res.clone())
            }
        }
        if let Some(ref parent) = self.parent {
            return parent.get(key)
        }
        None
    }

    fn insert(&mut self, key: String, value: ValueItem) {
        if self.map.is_none() {
            self.map = Some(HashMap::new())
        }
        self.map.as_mut().unwrap().insert(key, value);
    }
}


//------------ VarsStream ----------------------------------------------------

struct VarsStream {
    path: Path,
    builder: CollectionBuilder,
    parent: Vars,
    vars: Option<Vars>,
}

impl VarsStream {
    pub fn load(path: Path, parent: Vars, builder: &CollectionBuilder)
                -> Vars {
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                // Missing vars file is fine.
                return parent
            }
        };
        let mut data = Vec::new();
        if let Err(err) = file.read_to_end(&mut data) {
            builder.error((path, format!("Reading failed: {}", err)));
            return parent;
        }
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => {
                builder.error((path, format!("Not UTF-8 data.")));
                return parent;
            }
        };
        let mut stream = Self::new(path, parent, builder);
        if let Err(err) = yaml::GenericYamlLoader::load_from_str(&data,
                                                                 &mut stream) {
            stream.builder.error((stream.path.clone(), err));
        }
        stream.unwrap()
    }
}

impl VarsStream {
    fn new(path: Path, parent: Vars, builder: &CollectionBuilder) -> Self {
        VarsStream {
            path: path,
            builder: builder.clone(),
            parent: parent,
            vars: None,
        }
    }

    fn unwrap(self) -> Vars {
        match self.vars {
            Some(vars) => vars,
            None => self.parent,
        }
    }

    fn source(&self, mark: Option<Marker>) -> Source {
        match mark {
            Some(mark) => Source::in_file(self.path.clone(), mark),
            None => Source::file(self.path.clone())
        }
    }
}


impl yaml::Stream for VarsStream {
    type Item = ValueItem;
    type Sequence = SequenceBuilder;
    type Mapping = MappingBuilder;

    fn create_scalar(&self, value: &str, _style: TScalarStyle,
                     tag: &Option<TokenType>, mark: Marker) -> Self::Item {
        match parse_scalar(value, tag, &self.path, mark, &self.parent) {
            Ok(res) => res,
            Err(err) => {
                let pos = self.source(Some(mark));
                self.builder.error((pos, err));
                ValueItem::new(Value::Empty, self.path.clone(), Some(mark))
            }
        }
    }

    fn create_bad_value(&self) -> Self::Item {
        Item::new(Value::Empty, self.path.clone(), None)
    }

    fn create_sequence(&self, mark: Marker) -> Self::Sequence {
        SequenceBuilder::new(self.path.clone(), mark)
    }

    fn create_mapping(&self, mark: Marker) -> Self::Mapping {
        MappingBuilder::new(self.path.clone(), mark, self.builder.clone())
    }

    fn create_document(&mut self, item: Self::Item) {
        let (item, pos) = item.into_inner();
        if let Some(_) = self.vars {
            self.builder.error((pos,
                            String::from("there must only be one document")));
            return
        }
        let item = match item.into_mapping() {
            Some(item) => item,
            None => {
                self.builder.error((pos,
                                 String::from("document must be a mapping")));
                return
            }
        };
        let mut vars = Vars::new(Some(self.parent.clone()));
        for (key, value) in item {
            vars.insert(key, value)
        }
        self.vars = Some(vars);
    }
}

