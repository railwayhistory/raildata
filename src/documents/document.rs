//! The compound document.

use std::hash;
use ::load::construct::Context;
use ::load::yaml::Value;
use ::load::path;
use ::store::{ItemPermalink, Store, Variant};
use super::common::Common;
use super::types::{Key, Location, Marked};
use super::{Line, Organization, Path, Point, Source, Structure};


//------------ Document ------------------------------------------------------

/// This macro defines the `Document` type from a list of variants.
macro_rules! document {
    (
        $( $v:ident, )+
    ) => {

        pub enum Document {
            $( $v($v), )*
            Nonexisting(Nonexisting),
            Broken(Nonexisting),
        }

        impl Document {
            pub fn nonexisting(key: Key) -> Self {
                Document::Nonexisting(Nonexisting::new(key))
            }

            pub fn broken(key: Key) -> Self {
                Document::Broken(Nonexisting::new(key))
            }

            pub fn key(&self) -> &Key {
                match *self {
                    $( Document::$v(ref doc) => doc.key(), )*
                    Document::Nonexisting(ref doc) => doc.key(),
                    Document::Broken(ref doc) => doc.key(),
                }
            }

            pub fn is_nonexisting(&self) -> bool {
                if let Document::Nonexisting(_) = *self { true }
                else { false }
            }

            pub fn construct<C>(doc: Value, path: path::Path, context: &mut C)
                                -> Option<(Self, Key)>
                             where C: Context {
                let mut doc = match doc.into_mapping(context) {
                    Ok(doc) => doc,
                    Err(_) => return None,
                };
                let key: Key = match doc.take("key", context) {
                    Ok(key) => key,
                    Err(_) => return None,
                };
                let doctype: Marked<DocumentType>
                                = match doc.take("type", context) {
                    Ok(doctype) => doctype,
                    Err(_) => {
                        return Some((Document::broken(key.clone()), key))
                    }
                };
                let common = match Common::construct(&key, &mut doc, path,
                                                     context) {
                    Ok(common) => common,
                    Err(_) => {
                        return Some((Document::broken(key.clone()), key))
                    }
                };
                let doc = match doctype.to() {
                    $(
                        DocumentType::$v => {
                            $v::construct(common, doc, context)
                               .map(Document::$v)
                        }
                    )*
                };
                match doc {
                    Ok(doc) => Some((doc, key)),
                    Err(_) => Some((Document::broken(key.clone()), key)),
                }
            }

            pub fn location(&self) -> (path::Path, Location) {
                unimplemented!()
            }
        }


        impl PartialEq for Document {
            fn eq(&self, other: &Self) -> bool {
                self.key() == other.key()
            }
        }

        impl hash::Hash for Document {
            fn hash<H: hash::Hasher>(&self, state: &mut H) {
                self.key().hash(state)
            }
        }

        impl Variant for Document {
            type Item = Self;
            type Err = ();

            fn from_doc(doc: &Self) -> Result<&Self, ()> {
                Ok(doc)
            }

            fn from_doc_mut(doc: &mut Self) -> Result<&mut Self, ()> {
                Ok(doc)
            }
        }

        $(
            impl Variant for $v {
                type Item = Document;
                type Err = VariantError;

                fn from_doc(doc: &Document) -> Result<&$v, VariantError> {
                    match *doc {
                        Document::$v(ref doc) => Ok(doc),
                        _ => Err(VariantError(DocumentType::$v)),
                    }
                }

                fn from_doc_mut(doc: &mut Document)
                                -> Result<&mut $v, VariantError> {
                    match *doc {
                        Document::$v(ref mut doc) => Ok(doc),
                        _ => Err(VariantError(DocumentType::$v)),
                    }
                }
            }

            impl From<$v> for Document {
                fn from(var: $v) -> Document {
                    Document::$v(var)
                }
            }
        )*
    }
}

document! {
    Line,
    Organization,
    Path,
    Point,
    Source,
    Structure,
}


//------------ DocumentLink and DocumentStore --------------------------------

pub type DocumentLink = ItemPermalink<Document>;

pub type DocumentStore = Store<Document>;


//------------ Nonexisting ---------------------------------------------------

/// A document that doesn’t actually exist.
pub struct Nonexisting {
    key: Key
}

impl Nonexisting {
    pub fn new(key: Key) -> Self {
        Nonexisting { key }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }
}


//------------ DocumentType -------------------------------------------------

data_enum! {
    pub enum DocumentType {
        { Line: "line" }
        { Organization: "organization" }
        { Path: "path" }
        { Point: "point" }
        { Source: "source" }
        { Structure: "structure" }
    }
}


//------------ VariantError --------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VariantError(DocumentType);

