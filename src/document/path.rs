
use std::f64::INFINITY;
use std::str::FromStr;
use osmxml::elements::{MemberType, Osm, Relation};
use ::load::report::{self, Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::Mapping;
use ::store::{LoadStore, Stored, UpdateStore, PathLink, SourceLink};
use ::types::{IntoMarked, Location, Key, Marked};
use ::types::key::InvalidKey;
use super::common::{Common, Progress};


//------------ Path ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Path {
    common: Common,

    name: Option<String>,
    nodes: Vec<Node>,
    source: Vec<SourceLink>,

    node_names: Vec<(String, usize)>,
    node_descr: Vec<(usize, String)>,
}

impl Path {
    pub fn common(&self) -> &Common {
        &self.common
    }

    pub fn key(&self) -> &Key {
        self.common().key()
    }

    pub fn progress(&self) -> Progress {
        self.common().progress()
    }

    pub fn origin(&self) -> &Origin {
        &self.common().origin()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    pub fn nodes(&self) -> &[Node] {
        self.nodes.as_ref()
    }
}

impl<'a> Stored<'a, Path> {
    pub fn source(&self) -> Stored<'a, Vec<SourceLink>> {
        self.map(|item| &item.source)
    }
}

impl Path {
    fn new(key: Key, path: report::Path) -> Self {
        Path {
            common: Common::new(
                key.marked(Location::NONE),
                Progress::InProgress.marked(Location::NONE),
                Origin::new(path, Location::NONE)
            ),
            name: None,
            nodes: Vec::new(),
            source: Vec::new(),
            node_names: Vec::new(),
            node_descr: Vec::new(),
        }
    }

    pub fn from_yaml(
        _key: Marked<Key>,
        doc: Mapping,
        _context: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        report.error(PathInYaml.marked(doc.location()));
        Err(Failed)
    }

    pub fn from_osm(
        mut relation: Relation,
        osm: &Osm,
        documents: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<Self, Option<Key>> {
        if relation.tags().get("type") != Some("path") {
            report.unmarked_error(NonPathRelation(relation.id()));
            return Err(None)
        }
        let key = match relation.tags_mut().remove("key") {
            Some(key) => {
                match Key::from_string(key) {
                    Ok(key) => key,
                    Err(err) => {
                        report.unmarked_error(
                            InvalidRelationKey(relation.id(), err)
                        );
                        return Err(None);
                    }
                }
            }
            None => {
                report.unmarked_error(MissingKey(relation.id()));
                return Err(None);
            }
        };
        let mut path = Path::new(key.clone(), report.path());
        if let Err(_) = path.load_nodes(&mut relation, osm, report) {
            return Err(Some(key))
        }
        if let Err(_) = path.load_source(&mut relation, documents, report) {
            return Err(Some(key))
        }
        path.name = relation.tags_mut().remove("name");
        Ok(path)
    }

    fn load_nodes(
        &mut self,
        relation: &mut Relation,
        osm: &Osm,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut last_id = None;
        let mut last_tension = false; // last node has explicit post tension
        for member in relation.members() {
            if member.mtype() != MemberType::Way {
                report.unmarked_error(
                    NonWayMember { rel: relation.id(), target: member.id() }
                );
                continue;
            }
            let way = match osm.get_way(member.id()) {
                Some(way) => way,
                None => {
                    report.unmarked_error(
                        MissingWay { rel: relation.id(), way: member.id() }
                    );
                    continue
                }
            };
            let tension = match way.tags().get("type") {
                None => 1.,
                Some("curved") => 1.,
                Some("straight") => INFINITY,
                Some(value) => {
                    report.unmarked_warning(
                        IllegalWayType { way: way.id(), value: value.into() }
                    );
                    1.
                }
            };

            if way.nodes().is_empty() {
                report.unmarked_warning(EmptyWay(way.id()));
                continue;
            }
            let mut way_nodes = way.nodes().iter();
            if let Some(last) = last_id {
                let id = way_nodes.next().unwrap();
                if last != id {
                    report.unmarked_error(
                        NonContiguous {
                            rel: relation.id(),
                            way: way.id()
                        }
                    );
                    // That’s the end of this relation, really.
                    return Err(Failed)
                }
                if !last_tension {
                    self.nodes.last_mut().unwrap().post = tension;
                }
            }
            for id in way_nodes {
                let (node, name, descr, post_tension)
                    = Self::load_node(*id, osm, tension, report)?;
                if let Some(name) = name {
                    self.node_names.push((name, self.nodes.len()));
                }
                if let Some(descr) = descr {
                    self.node_descr.push((self.nodes.len(), descr));
                }
                self.nodes.push(node);
                last_tension = post_tension;
                last_id = Some(id);
            }
        }
        self.node_names.sort_by(|x, y| x.0.cmp(&y.0));

        let mut remaining: &[_] = self.node_names.as_ref();
        while remaining.len() > 1 {
            if remaining[0].0 == remaining[1].0 {
                report.unmarked_error(
                    DuplicateName(remaining[0].0.clone())
                );
            }
            remaining = &remaining[1..]
        }

        Ok(())
    }

    fn load_node(
        id: i64,
        osm: &Osm,
        tension: f64,
        report: &mut PathReporter
    ) -> Result<(Node, Option<String>, Option<String>, bool), Failed> {
        let node = match osm.get_node(id) {
            Some(node) => node,
            None => {
                report.unmarked_error(MissingNode(id));
                return Err(Failed)
            }
        };
        let pre = match node.tags().get("pre") {
            Some(pre) => match Self::load_f64(pre) {
                Some(pre) => pre,
                None => {
                    report.unmarked_warning(InvalidPre(id));
                    tension
                }
            },
            None => tension
        };
        let (post, have_post) = match node.tags().get("post") {
            Some(post) => match Self::load_f64(post) {
                Some(post) => (post, true),
                None => {
                    report.unmarked_warning(InvalidPost(id));
                    (tension, false)
                }
            },
            None => (tension, false)
        };
        let name = node.tags().get("name").map(String::from);
        let description = node.tags().get("description").map(String::from);
        Ok((
            Node::new(node.lon(), node.lat(), pre, post),
            name,
            description,
            have_post
        ))
    }

    fn load_f64(value: &str) -> Option<f64> {
        f64::from_str(value).ok()
    }

    fn load_source(
        &mut self,
        relation: &mut Relation,
        documents: &mut LoadStore,
        report: &mut PathReporter
    ) -> Result<(), Failed> {
        let source = match relation.tags_mut().remove("source") {
            Some(source) => source,
            None => return Ok(())
        };
        for item in source.split_whitespace() {
            let key = match Key::from_str(item) {
                Ok(key) => key,
                Err(err) => {
                    report.unmarked_error(err);
                    return Err(Failed)
                }
            };
            let key = key.marked(Location::NONE);
            if let Ok(link) = SourceLink::forge(key, documents, report) {
                self.source.push(link.into_value())
            }
            else {
                return Err(Failed)
            }
        }
        Ok(())
    }

    pub fn crosslink(
        &self,
        _link: PathLink,
        _store: &mut UpdateStore,
        _report: &mut StageReporter
    ) {
    }

    pub fn verify(&self, _report: &mut StageReporter) {
    }
}


//------------ Node ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Node {
    lon: f64,
    lat: f64,
    pre: f64,
    post: f64,
}

impl Node {
    pub fn new(lon: f64, lat: f64, pre: f64, post: f64) -> Self {
        Node { lon, lat, pre, post }
    }
}


//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="path documents in YAML files currently not supported")]
pub struct PathInYaml;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="relation {} is not a path", _0)]
pub struct NonPathRelation(i64);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="relation {} is missing the 'key' attribute", _0)]
pub struct MissingKey(i64);

#[derive(Clone, Debug, Fail)]
#[fail(display="relation {}: {}", _0, _1)]
pub struct InvalidRelationKey(i64, InvalidKey);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="relation {} has non-way member {}", rel, target)]
pub struct NonWayMember {
    rel: i64,
    target: i64,
}

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="relation {} references non-exisitng way {}", rel, way)]
pub struct MissingWay {
    rel: i64,
    way: i64,
}

#[derive(Clone, Debug, Fail)]
#[fail(display="way {} has invalid type '{}'", way, value)]
pub struct IllegalWayType {
    way: i64,
    value: String,
}

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="way {} is empty", _0)]
pub struct EmptyWay(i64);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="relation {} is non-contiguous at way {}", rel, way)]
pub struct NonContiguous {
    rel: i64,
    way: i64
}

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="missing node {}", _0)]
pub struct MissingNode(i64);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid pre tag in node {}", _0)]
pub struct InvalidPre(i64);

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display="invalid post tag in node {}", _0)]
pub struct InvalidPost(i64);

#[derive(Clone, Debug, Fail)]
#[fail(display="duplicate node name '{}'", _0)]
pub struct DuplicateName(String);


