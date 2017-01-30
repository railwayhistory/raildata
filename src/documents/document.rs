use ::collection::CollectionBuilder;
use ::load::error::ErrorGatherer;
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
    pub fn from_yaml(item: ValueItem, collection: &mut CollectionBuilder,
                     errors: &ErrorGatherer) -> Result<Self, ()> {
        let mut item = item.into_mapping(errors)?;
        let key = item.mandatory_key("key", errors)?
                      .into_string(errors)?;
        let doctype = item.parse::<DocumentType>("type", collection, errors)?;
        match doctype {
            DocumentType::Line => {
                Ok(Document::Line(Line::from_yaml(key, item, collection,
                                                  errors)?))
            }
            DocumentType::Organization => {
                Ok(Document::Organization(Organization::from_yaml(key, item,
                                                                  collection,
                                                                  errors)?))
            }
            DocumentType::Path => {
                Ok(Document::Path(Path::from_yaml(key, item, collection,
                                                  errors)?))
            }
            DocumentType::Point => {
                Ok(Document::Point(Point::from_yaml(key, item, collection,
                                                    errors)?))
            }
            DocumentType::Source => {
                Ok(Document::Source(Source::from_yaml(key, item,
                                                      collection, errors)?))
            }
            DocumentType::Structure => {
                Ok(Document::Structure(Structure::from_yaml(key, item,
                                                            collection,
                                                            errors)?))
            }
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

