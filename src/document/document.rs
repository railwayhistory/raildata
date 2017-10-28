//! The compound document.

use std::{fmt, hash};
use ::types::{Key, Location, Marked};
use ::load::path;
use ::load::construct::{ConstructContext, Failed};
use ::load::yaml::{Value};
use super::{Line, Organization, Path, Point, Source, Structure};
use super::broken::Broken;
use super::nonexisting::Nonexisting;


//------------ Document ------------------------------------------------------

macro_rules! document {  ( $( $v:ident, )* ) => {

    pub enum Document {
        $(
            $v($v),
        )*
        Nonexisting(Nonexisting),
        Broken(Broken),
    }

    impl Document {
        pub fn nonexisting(key: Key) -> Self {
            Document::Nonexisting(Nonexisting::new(key))
        }

        pub fn broken(key: Marked<Key>, doc_type: Option<DocumentType>)
                      -> Self {
            Document::Broken(Broken::new(key, doc_type))
        }

        pub fn construct(doc: Value, context: &mut ConstructContext)
                         -> Result<(Self, Key), Failed> {
            let mut doc = doc.into_mapping(context)?;
            let key: Marked<Key> = doc.take("key", context)?;
            let doc_type = match doc.take("type", context) {
                Ok(doc_type) => doc_type,
                Err(_) => {
                    return Ok((Self::broken(key.clone(), None),
                               key.into_value()))
                }
            };
            let res = match doc_type {
                $(
                    DocumentType::$v => {
                        $v::construct(key.clone(), doc, context)
                           .map(Document::$v)
                    }
                )*
            };
            let res = match res {
                Ok(res) => res,
                Err(_) => Self::broken(key.clone(), Some(doc_type)),
            };
            Ok((res, key.into_value()))
        }
    }

    impl Document {
        pub fn key(&self) -> &Key {
            match *self {
                $(
                    Document::$v(ref doc) => doc.key(),
                )*
                Document::Nonexisting(ref doc) => doc.key(),
                Document::Broken(ref doc) => doc.key(),
            }
        }

        pub fn doc_type(&self) -> Option<DocumentType> {
            match *self {
                $(
                    Document::$v(_) => Some(DocumentType::$v),
                )*
                Document::Broken(ref doc) => doc.doc_type(),
                _ => None,
            }
        }

        pub fn is_nonexisting(&self) -> bool {
            match *self {
                Document::Nonexisting(_) => true,
                _ => false
            }
        }

        pub fn as_nonexisting(&self) -> Option<&Nonexisting> {
            match *self {
                Document::Nonexisting(ref doc) => Some(doc),
                _ => None
            }
        }

        pub fn as_nonexisting_mut(&mut self) -> Option<&mut Nonexisting> {
            match *self {
                Document::Nonexisting(ref mut doc) => Some(doc),
                _ => None
            }
        }

        pub fn is_broken(&self) -> bool {
            match *self {
                Document::Broken(_) => true,
                _ => false
            }
        }

        pub fn as_broken(&self) -> Option<&Broken> {
            match *self {
                Document::Broken(ref doc) => Some(doc),
                _ => None
            }
        }

        pub fn location(&self) -> Option<(&path::Path, Location)> {
            match *self {
                $(
                    Document::$v(ref doc) => Some(doc.location()),
                )*
                _ => None,
            }
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

    $(
        impl Variant for $v {
            const DOC_TYPE: DocumentType = DocumentType::$v;

            fn from_doc(doc: &Document) -> Result<&Self, VariantError> {
                match *doc {
                    Document::$v(ref doc) => Ok(doc),
                    _ => Err(VariantError { want: DocumentType::$v,
                                            got: doc.doc_type() }),
                }
            }

            fn from_doc_mut(doc: &mut Document)
                            -> Result<&mut Self, VariantError> {
                match *doc {
                    Document::$v(ref mut doc) => Ok(doc),
                    _ => Err(VariantError { want: DocumentType::$v,
                                            got: doc.doc_type() }),
                }
            }
        }
    )*
}}

document! {
    Line,
    Organization,
    Path,
    Point,
    Source,
    Structure,
}

//------------ DocumentType --------------------------------------------------

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


//------------ Variant ------------------------------------------------------

pub trait Variant: Sized {
    const DOC_TYPE: DocumentType;
    fn from_doc(doc: &Document) -> Result<&Self, VariantError>;
    fn from_doc_mut(doc: &mut Document) -> Result<&mut Self, VariantError>;
}


//------------ VariantError --------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VariantError {
    want: DocumentType,
    got: Option<DocumentType>,
}

impl VariantError {
    pub fn new(want: DocumentType, got: Option<DocumentType>) -> Self {
        VariantError { want, got }
    }
}

impl fmt::Display for VariantError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.got {
            Some(got) => write!(f, "expected {} document, got {}",
                                self.want, got),
            None => write!(f, "broken link to {} document", self.want),
        }
    }
}

