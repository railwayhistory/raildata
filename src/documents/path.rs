use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::ShortVec;
use super::document::{Document, DocumentType};
use super::point::PointRef;


//------------ Path ----------------------------------------------------------

pub struct Path {
    key: String,
    nodes: Vec<Node>,
    name: Option<String>
}

impl Path {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }
}

impl Path {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let nodes = item.parse_mandatory("nodes", builder);
        let name = item.parse_opt("name", builder);
        try_key!(item.exhausted(builder), key);
        Ok(Document::Path(Path {
            nodes: try_key!(nodes, key),
            name: try_key!(name, key),
            key: key,
        }))
    }
}


//------------ Node ---------------------------------------------------------

pub struct Node {
    lon: f64,
    lat: f64,
    pre: f64,
    post: f64,
    name: Option<String>,
    point: ShortVec<PointRef>,
    description: Option<String>,
}

impl Node {
    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn pre(&self) -> f64 {
        self.pre
    }

    pub fn post(&self) -> f64 {
        self.post
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    pub fn point(&self) -> &ShortVec<PointRef> {
        &self.point
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }
}

impl FromYaml for Node {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let mut item = item.into_mapping(builder)?;
        let lon = item.parse_mandatory("lon", builder);
        let lat = item.parse_mandatory("lat", builder);
        let pre = item.parse_mandatory("pre", builder);
        let post = item.parse_mandatory("post", builder);
        let name = item.parse_opt("name", builder);
        let point = item.parse_opt("point", builder);
        let description = item.parse_opt("description", builder);
        item.exhausted(builder)?;

        Ok(Node {
            lon: lon?, lat: lat?, pre: pre?, post: post?,
            name: name?,
            point: point?.unwrap_or(ShortVec::Empty),
            description: description?
        })
    }
}


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
                                   Some(DocumentType::Path))))
    }
}

