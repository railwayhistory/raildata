use std::f64::INFINITY;
use osmxml::elements as osm;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::path;
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

    pub fn from_osm(mut relation: osm::Relation, osm: &osm::Osm,
                    path: &path::Path, builder: &CollectionBuilder)
                    -> Result<Document, Option<String>> {
        if relation.tags().get("type") != Some("path") {
            builder.str_warning(path, "contains a non-path relation");
            return Err(None)
        }
        let key = match relation.tags_mut().remove("key") {
            Some(key) => key,
            None => {
                builder.str_error(path, "missing 'key' in path relation");
                return Err(None);
            }
        };
        let name = relation.tags_mut().remove("name");

        let mut nodes = Vec::<Node>::new();
        let mut last_id = None;
        for member in relation.members() {
            if member.mtype() != osm::MemberType::Way {
                builder.error((path.clone(),
                               format!("non-way member in relation {}",
                                       &key)));
                return Err(Some(key));
            }
            let way = match osm.get_way(member.id()) {
                Some(way) => way,
                None => {
                    builder.error((path.clone(),
                                   format!("missing way {} referenced in {}",
                                           member.id(), &key)));
                    return Err(Some(key));
                }
            };
            let tension = match way.tags().get("type") {
                None => 1.,
                Some("curved") => 1.,
                Some("straight") => INFINITY,
                Some(other) => {
                    builder.error((path.clone(),
                                  format!("invalid type value '{}' in way {}",
                                          other, way.id())));
                    return Err(Some(key));
                }
            };
            let mut way_nodes = way.nodes().iter();
            match way_nodes.next() {
                None => continue,
                Some(id) => {
                    let id = *id;
                    if let Some(last) = last_id {
                        if last != id {
                            builder.error((path.clone(),
                                           format!("non-contiguous path {}",
                                                   &key)));
                            return Err(Some(key));
                        }
                        // XXX This will overwrite an explicit post tension.
                        nodes.last_mut().unwrap().post = tension;
                    }
                    else {
                        let node = match Node::from_osm(id, &osm, tension,
                                                        path, builder) {
                            Ok(node) => node,
                            Err(()) => return Err(Some(key)),
                        };
                        nodes.push(node);
                        last_id = Some(id);
                    }
                }
            }
            for id in way_nodes {
                let id = *id;
                let node = match Node::from_osm(id, &osm, tension, path,
                                                builder) {
                    Ok(node) => node,
                    Err(()) => return Err(Some(key)),
                };
                nodes.push(node);
                last_id = Some(id);
            }
        }
        Ok(Document::Path(Path {
            key: key,
            nodes: nodes,
            name: name
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

impl Node {
    fn from_osm(id: i64, osm: &osm::Osm, tension: f64, path: &path::Path,
                builder: &CollectionBuilder) -> Result<Self, ()> {
        let node = match osm.get_node(id) {
            Some(node) => node,
            None => {
                builder.error((path.clone(), format!("Missing node {}", id)));
                return Err(())
            }
        };
        let pre = match node.tags().get("pre") {
            Some(pre) => into_f64(pre, path, builder)?,
            None => tension
        };
        let post = match node.tags().get("post") {
            Some(post) => into_f64(post, path, builder)?,
            None => tension
        };
        let name = node.tags().get("name").map(String::from);
        let point = Self::make_point(node.tags().get("point"), path, builder)?;
        let description = node.tags().get("description").map(String::from);
        Ok(Node {
            lon: node.lon(), lat: node.lat(), pre: pre, post: post,
            name: name, point: point, description: description
        })
    }

    fn make_point(s: Option<&str>, path: &path::Path,
                  builder: &CollectionBuilder)
                  -> Result<ShortVec<PointRef>, ()> {
        if let Some(s) = s {
            let mut parts = s.split_whitespace();
            let first = match parts.next() {
                Some(first) => first,
                None => return Ok(ShortVec::Empty),
            };
            let first = PointRef::new(builder, first, path.into());
            let second = match parts.next() {
                Some(second) => second,
                None => return Ok(ShortVec::One(first)),
            };
            let second = PointRef::new(builder, second, path.into());
            let mut vec = vec![first, second];
            for item in parts {
                let item = PointRef::new(builder, item, path.into());
                vec.push(item);
            }
            Ok(ShortVec::Many(vec))
        }
        else {
            Ok(ShortVec::Empty)
        }
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


//------------ Helpers -------------------------------------------------------

fn into_f64(s: &str, path: &path::Path, builder: &CollectionBuilder)
            -> Result<f64, ()> {
    use std::str::FromStr;

    f64::from_str(s).map_err(|_| {
        builder.error((path.clone(),
                       format!("Illegal float value '{}'", s)));
        ()
    })
}
