use std::{fmt, io, mem};
use std::collections::HashSet;
use std::f64::INFINITY;
use osmxml::elements::{MemberType, Osm, Relation};
use osmxml::read::read_xml;
use ::documents::document::Document;
use ::documents::links::{PointLink, SourceLink};
use ::documents::path::{Path, Node};
use ::documents::types::{Key, Location};
use super::construct::Context;
use super::tree::TreeContext;


//------------ load_osm_file -------------------------------------------------

pub fn load_osm_file<R: io::Read>(read: &mut R, context: &mut TreeContext) {
    let mut osm = match read_xml(read) {
        Ok(osm) => osm,
        Err(err) => {
            context.push_error((err, Location::NONE));
            return;
        }
    };
    
    // Swap out the relations so we don’t hold a mutable reference to
    // `osm` while draining the relations.
    let mut relations = HashSet::new();
    mem::swap(osm.relations_mut(), &mut relations);
    for relation in relations.drain() {
        load_relation(relation, &osm, context)
    }
}


fn load_relation(mut relation: Relation, osm: &Osm,
                 context: &mut TreeContext) {
    if relation.tags().get("type") != Some("path") {
        push_error(context, OsmError::NonPathRelation(relation.id()));
        return
    }
    let key: Key = match relation.tags_mut().remove("key") {
        Some(key) => key.into(),
        None => {
            push_error(context, OsmError::MissingKey(relation.id()));
            return;
        }
    };
    let nodes = match load_nodes(&relation, osm, context) {
        None => {
            context.insert_document(key.clone(), Document::broken(key));
            return;
        }
        Some(nodes) => nodes
    };
    let name = relation.tags_mut().remove("name");
    let source = load_source(relation.tags_mut().remove("source").as_ref()
                                     .map(AsRef::as_ref),
                             context);

    context.insert_document(key.clone(),
                            Document::Path(Path::new(key, name, nodes,
                                                     source)))
}

fn load_nodes(relation: &Relation, osm: &Osm, context: &mut TreeContext)
              -> Option<Vec<Node>> {
    let mut nodes = Vec::<Node>::new();
    let mut last_id = None;
    for member in relation.members() {
        if member.mtype() != MemberType::Way {
            push_error(context,
                       OsmError::NonWayMember { rel: relation.id(),
                                                target: member.id() });
            continue;
        }
        let way = match osm.get_way(member.id()) {
            Some(way) => way,
            None => {
                push_error(context, OsmError::MissingWay { rel: relation.id(),
                                                           way: member.id() });
                continue
            }
        };
        let tension = match way.tags().get("type") {
            None => 1.,
            Some("curved") => 1.,
            Some("straight") => INFINITY,
            Some(value) => {
                push_error(context,
                           OsmError::IllegalType { way: way.id(),
                                                   value: value.into() });
                1.
            }
        };

        let mut way_nodes = way.nodes().iter();
        match way_nodes.next() {
            None => {
                push_error(context, OsmError::EmptyWay(way.id()));
                continue
            }
            Some(id) => {
                if let Some(last) = last_id {
                    if last != id {
                        push_error(context,
                                   OsmError::NonContiguous {
                                       rel: relation.id(),
                                       way: way.id()
                                   });
                        // That’s the end of this relation, really.
                        return None;
                    }
                    // XXX This will overwrite an explicit post tension.
                    nodes.last_mut().unwrap().set_post(tension);
                }
                else {
                    let node = match load_node(*id, osm, tension, context) {
                        None => return None,
                        Some(node) => node
                    };
                    nodes.push(node);
                    last_id = Some(id);
                }
            }
        }
        for id in way_nodes {
            let node = match load_node(*id, osm, tension, context) {
                None => return None,
                Some(node) => node,
            };
            nodes.push(node);
            last_id = Some(id);
        }
    }
    Some(nodes)
}


fn load_node(id: i64, osm: &Osm, tension: f64, context: &mut TreeContext)
             -> Option<Node> {
    let node = match osm.get_node(id) {
        Some(node) => node,
        None => {
            push_error(context, OsmError::MissingNode(id));
            return None
        }
    };
    let pre = match node.tags().get("pre") {
        Some(pre) => match load_f64(pre) {
            Some(pre) => pre,
            None => {
                push_error(context, OsmError::InvalidPre(id));
                tension
            }
        },
        None => tension
    };
    let post = match node.tags().get("post") {
        Some(post) => match load_f64(post) {
            Some(post) => post,
            None => {
                push_error(context, OsmError::InvalidPost(id));
                tension
            }
        },
        None => tension
    };
    let name = node.tags().get("name").map(String::from);
    let point = load_point(node.tags().get("point"), context);
    let description = node.tags().get("description").map(String::from);
    Some(Node::new(node.lon(), node.lat(), pre, post, name, point,
                   description))
}

fn load_f64(value: &str) -> Option<f64> {
    use std::str::FromStr;

    f64::from_str(value).ok()
}

fn load_point(value: Option<&str>, context: &mut TreeContext)
              -> Option<Vec<PointLink>> {
    value.map(|value| {
        value.split_whitespace().map(|key| {
            PointLink::from_value(context.get_link(&String::from(key).into()))
        }).collect()
    })
}

fn load_source(value: Option<&str>, context: &mut TreeContext)
               -> Option<Vec<SourceLink>> {
    value.map(|value| {
        value.split_whitespace().map(|key| {
            SourceLink::from_value(context.get_link(&String::from(key).into()))
        }).collect()
    })
}


//------------ Helpers -------------------------------------------------------

fn push_error(context: &mut TreeContext, err: OsmError) {
    context.push_error((err, Location::NONE))
}


//------------ OsmError ------------------------------------------------------

#[derive(Clone, Debug)]
pub enum OsmError {
    NonPathRelation(i64),
    MissingKey(i64),
    NonWayMember { rel: i64, target: i64 },
    MissingWay { rel: i64, way: i64 },
    IllegalType { way: i64, value: String },
    EmptyWay(i64),
    NonContiguous { rel: i64, way: i64 },
    MissingNode(i64),
    InvalidPre(i64),
    InvalidPost(i64),
}

impl fmt::Display for OsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::OsmError::*;

        match *self {
            NonPathRelation(id) => write!(f, "relation {} in not a path", id),
            MissingKey(id) => write!(f, "realtion {} is missing the key", id),
            NonWayMember { rel, target } => {
                write!(f, "relation {} has non-way member (ref: {})",
                       rel, target)
            }
            MissingWay { rel, way } => {
                write!(f, "relation {} references non-exisiting way {}",
                       rel, way)
            }
            IllegalType { way, ref value } => {
                write!(f, "way {} has invalid type '{}'", way, value)
            }
            EmptyWay(id) => write!(f, "way {} is empty", id),
            NonContiguous { rel, way } => {
                write!(f, "relation {} is non-contiguous at way {}", rel, way)
            }
            MissingNode(id) => write!(f, "missing node {}", id),
            InvalidPre(id) => write!(f, "invalid pre tag in node {}", id),
            InvalidPost(id) => write!(f, "invalid post tag in node {}", id),
        }
    }
}
