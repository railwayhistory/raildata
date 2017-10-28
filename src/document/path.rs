use std::fmt;
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::crosslink::CrosslinkContext;
use ::load::path;
use ::load::yaml::{Mapping, Value};
use ::links::{DocumentLink, PointLink, SourceLink};
use ::types::{Key, Location, Marked};


//------------ Path ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Path {
    key: Key,
    path: path::Path,
    location: Location,

    name: Option<String>,
    nodes: Vec<Node>,
    source: Option<Vec<SourceLink>>,
}

impl Path {
    pub fn key(&self) -> &Key { &self.key }
    pub fn location(&self) -> (&path::Path, Location) {
        (&self.path, self.location)
    }
    pub fn name(&self) -> Option<&str> { self.name.as_ref().map(AsRef::as_ref) }
    pub fn nodes(&self) -> &[Node] { &self.nodes }
    pub fn source(&self) -> Option<&[SourceLink]> { 
        self.source.as_ref().map(AsRef::as_ref)
    }
}

impl Path {
    pub fn new(key: Key, name: Option<String>, nodes: Vec<Node>,
               source: Option<Vec<SourceLink>>, path: path::Path,
               location: Location) -> Self {
        Path { key, path, location, name, nodes, source }
    }

    pub fn construct(key: Marked<Key>, mut doc: Marked<Mapping>,
                     context: &mut ConstructContext) -> Result<Self, Failed> {
        let name = doc.take_opt("name", context);
        let nodes = doc.take("nodes", context);
        let source = doc.take_default("source", context);
        let location = doc.location();
        doc.exhausted(context)?;
        Ok(Path {
            key: key.into_value(),
            path: context.path().clone(),
            location,
            name: name?,
            nodes: nodes?,
            source: source?,
        })
    }

    pub fn crosslink(&self, _link: DocumentLink,
                     _context: &mut CrosslinkContext) {
    }
}


//------------ Node ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Node {
    lon: f64,
    lat: f64,
    pre: f64,
    post: f64,
    extra: Option<Box<NodeExtra>>,
}

#[derive(Clone, Debug)]
struct NodeExtra {
    name: Option<String>,
    point: Vec<PointLink>,
    description: Option<String>,
}

impl Node {
    pub fn new(lon: f64, lat: f64, pre: f64, post: f64, name: Option<String>,
               point: Option<Vec<PointLink>>, description: Option<String>)
               -> Self {
        Node {
            lon, lat, pre, post,
            extra: {
                if name.is_some() || point.is_some() || description.is_some() {
                    let point = point.unwrap_or_default();
                    Some(Box::new(NodeExtra { name, point, description }))
                }
                else {
                    None
                }
            }
        }
    }
}

impl Constructable for Node {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let lon = value.take("lon", context);
        let lat = value.take("lat", context);
        let pre = value.take("pre", context);
        let post = value.take("post", context);
        let name = value.take_opt("name", context);
        let point = value.take_default("point", context);
        let description = value.take_opt("description", context);
        value.exhausted(context)?;
        let name = name?;
        let point: Vec<_> = point?;
        let description = description?;
        let extra = if name.is_some() || !point.is_empty()
                        || description.is_some() {
            Some(Box::new(NodeExtra { name, point, description }))
        }
        else {
            None
        };
        Ok(Node {
            lon: lon?,
            lat: lat?,
            pre: pre?,
            post: post?,
            extra
        })
    }
}

impl Node {
    pub fn lon(&self) -> f64 { self.lon }
    pub fn lat(&self) -> f64 { self.lat }
    pub fn pre(&self) -> f64 { self.pre }
    pub fn post(&self) -> f64 { self.post }
    pub fn name(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|extra| {
            extra.name.as_ref().map(AsRef::as_ref)
        })
    }
    pub fn point(&self) -> Option<&[PointLink]> {
        self.extra.as_ref().and_then(|extra| Some(extra.point.as_ref()))
    }
    pub fn description(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|extra| {
            extra.description.as_ref().map(AsRef::as_ref)
        })
    }

    pub fn set_post(&mut self, post: f64) {
        self.post = post
    }
}

/*
impl Constructable for Node {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        let mut value = value.into_mapping(context)?;
        let lon = value.take("lon", context);
        let lat = value.take("lat", context);
        let pre = value.take("pre", context);
        let post = value.take("post", context);
        let name = value.take_opt("name", context);
        let point: Result<List<PointLink>, _>
                    = value.take_default("point", context);
        let description = value.take_opt("description", context);
        value.exhausted(context)?;
        let name = name?;
        let point = point?;
        let description = description?;
        let extra = if name.is_some() || !point.is_empty() ||
                       description.is_some() {
            Some(Box::new(NodeExtra { name, point, description }))
        }
        else {
            None
        };
        Ok(Node {
            lon: lon?,
            lat: lat?,
            pre: pre?,
            post: post?,
            extra: extra
        })
    }
}
*/


#[derive(Clone, Copy, Debug)]
struct PathFromYaml;

impl fmt::Display for PathFromYaml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "path document not allowed in yaml source")
    }
}
