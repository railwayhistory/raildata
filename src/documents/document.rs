use ::collection::CollectionBuilder;
use ::load::yaml::{ValueItem}; 
use super::line::Line;
use super::organization::Organization;
use super::path::Path;
use super::point::Point;
use super::source::Source;
use super::structure::Structure;


//------------ Document ------------------------------------------------------

pub enum Document {
    Line(Line),
    Organization(Organization),
    Path(Path),
    Point(Point),
    Source(Source),
    Structure(Structure),
}

impl Document {
    /// Parses a document from its YAML representation.
    ///
    /// Returns `Ok(doc)` if all went well. Returns `Err(Some(key))` if
    /// parsing failed but the YAML representation contained a key, so there
    /// is at least in theory a document with that key. Returns `Err(None)`
    /// if all is hopeless.
    pub fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                     -> Result<Self, Option<String>> {
        let mut item = item.into_mapping(builder).map_err(|_| None)?;
        let key = item.parse_mandatory("key", builder)
                      .map_err(|_| None)?;
        let doctype = try_key!(item.parse_mandatory::<DocumentType>("type",
                                                                    builder),
                               key);
        match doctype {
            DocumentType::Line => Line::from_yaml(key, item, builder),
            DocumentType::Organization
                => Organization::from_yaml(key, item, builder),
            DocumentType::Path => Path::from_yaml(key, item, builder),
            DocumentType::Point => Point::from_yaml(key, item, builder),
            DocumentType::Source => Source::from_yaml(key, item, builder),
            DocumentType::Structure
                => Structure::from_yaml(key, item, builder),
        }
    }

    pub fn key(&self) -> &str {
        match *self {
            Document::Line(ref doc) => doc.key(),
            Document::Organization(ref doc) => doc.key(),
            Document::Path(ref doc) => doc.key(),
            Document::Point(ref doc) => doc.key(),
            Document::Source(ref doc) => doc.key(),
            Document::Structure(ref doc) => doc.key(),
        }
    }

    pub fn doc_type(&self) -> DocumentType {
        match *self {
            Document::Line(_) => DocumentType::Line,
            Document::Organization(_) => DocumentType::Organization,
            Document::Path(_) => DocumentType::Path,
            Document::Point(_) => DocumentType::Point,
            Document::Source(_) => DocumentType::Source,
            Document::Structure(_) => DocumentType::Structure,
        }
    }
}

impl AsRef<Line> for Document {
    fn as_ref(&self) -> &Line {
        match *self {
            Document::Line(ref doc) => doc,
            _ => panic!("not an organization")
        }
    }
}

impl AsRef<Organization> for Document {
    fn as_ref(&self) -> &Organization {
        match *self {
            Document::Organization(ref doc) => doc,
            _ => panic!("not an organization")
        }
    }
}

impl AsRef<Path> for Document {
    fn as_ref(&self) -> &Path {
        match *self {
            Document::Path(ref doc) => doc,
            _ => panic!("not an organization")
        }
    }
}

impl AsRef<Point> for Document {
    fn as_ref(&self) -> &Point{
        match *self {
            Document::Point(ref doc) => doc,
            _ => panic!("not an organization")
        }
    }
}

impl AsRef<Source> for Document {
    fn as_ref(&self) -> &Source {
        match *self {
            Document::Source(ref source) => source,
            _ => panic!("not a source")
        }
    }
}

impl AsRef<Structure> for Document {
    fn as_ref(&self) -> &Structure {
        match *self {
            Document::Structure(ref source) => source,
            _ => panic!("not a source")
        }
    }
}


//------------ DocumentType --------------------------------------------------

mandatory_enum! {
    pub enum DocumentType {
        (Organization => "organization"),
        (Line => "line"),
        (Path => "path"),
        (Point => "point"),
        (Source => "source"),
        (Structure => "structure"),
    }
}

