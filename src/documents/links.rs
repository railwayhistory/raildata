use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::Value;
use ::store::{Link, Variant};
use super::{Document, Line, Organization, Path, Point, Source};
use super::types::{Key, Marked};


pub type DocumentLink = Link<Document>;

pub type LineLink = Link<Line>;

pub type OrganizationLink = Link<Organization>;

pub type PathLink = Link<Path>;

pub type PointLink = Link<Point>;

pub type SourceLink = Link<Source>;


impl<T: Variant<Item=Document>> Constructable for Marked<Link<T>> {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        Key::construct(value, context).map(|key| {
            let location = key.location();
            Marked::new(context.get_link(&key), location)
        })
    }
}

