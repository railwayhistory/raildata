use crate::library::{Library, LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::Link;
use crate::types::{Key, Location, Marked};
use super::{Line, Organization, Path, Point, Source, Structure};
use super::common::DocumentType;

macro_rules! document { ( $( ($vattr:ident, $vtype:ident,
                              $vlink:ident ), )* ) => {

    //------------ Document --------------------------------------------------

    #[derive(Clone, Debug, From)]
    pub enum Document {
        $(
            $vtype($vtype),
        )*
    }

    impl Document {
        pub fn key(&self) -> &Key {
            match *self {
                $(
                    Document::$vtype(ref inner) => inner.key(),
                )*
            }
        }

        pub fn doctype(&self) -> DocumentType {
            match *self {
                $(
                    Document::$vtype(_) => DocumentType::$vtype,
                )*
            }
        }

        pub fn origin(&self) -> &Origin {
            match *self {
                $(
                    Document::$vtype(ref inner) => inner.origin(),
                )*
            }
        }

        pub fn location(&self) -> Location {
            self.origin().location()
        }
    }

    impl Document {
        pub fn from_yaml(
            key: Marked<Key>,
            doctype: DocumentType,
            doc: Mapping,
            context: &LibraryBuilder,
            report: &mut PathReporter
        ) -> Result<Self, Failed> {
            match doctype {
                $(
                    DocumentType::$vtype => {
                        $vtype::from_yaml(key, doc, context, report)
                            .map(Document::$vtype)
                    }
                )*
            }
        }

        pub fn crosslink(
            &self,
            link: DocumentLink,
            library: &LibraryMut,
            report: &mut StageReporter
        ) {
            match *self {
                $(
                    Document::$vtype(ref inner) => {
                        inner.crosslink(link.into(), library, report)
                    }
                )*
            }
        }
    }


    //------------ Links -----------------------------------------------------

    $(
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $vlink(Link<Document>);

        impl $vlink {
            pub fn build(
                key: Marked<Key>,
                store: &LibraryBuilder,
                report: &mut PathReporter,
            ) -> Marked<Self> {
                store.build_link(key, Some(DocumentType::$vtype), report)
                    .map(Into::into)
            }

            pub fn follow(self, library: &Library) -> &$vtype {
                match *self.0.follow(library.store()) {
                    Document::$vtype(ref inner) => inner,
                    _ => panic!("link to wrong document type")
                }
            }

            pub fn update<F>(self, library: &LibraryMut, op: F)
            where F: Fn(&mut $vtype) + 'static + Send {
                library.update(self.into(), move |document| {
                    match *document {
                        Document::$vtype(ref mut inner) => op(inner),
                        _ => panic!("link to wrong document type")
                    }
                })
            }
        }

        impl From<DocumentLink> for $vlink {
            fn from(link: DocumentLink) -> $vlink {
                $vlink(link.0)
            }
        }

        impl From<$vlink> for DocumentLink {
            fn from(link: $vlink) -> DocumentLink {
                DocumentLink(link.0)
            }
        }

        impl FromYaml<LibraryBuilder> for Marked<$vlink> {
            fn from_yaml(
                value: Value,
                context: &LibraryBuilder,
                report: &mut PathReporter
            ) -> Result<Self, Failed> {
                Ok($vlink::build(
                    Marked::from_yaml(value, context, report)?,
                    context,
                    report
                ))
            }
        }
    )*

    //-------- DocumentLink --------------------------------------------------

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct DocumentLink(Link<Document>);

    impl DocumentLink {
        pub fn build(
            key: Marked<Key>,
            store: &LibraryBuilder,
            report: &mut PathReporter,
        ) -> Marked<Self> {
            store.build_link(key, None, report).map(Into::into)
        }

        pub fn update<F>(self, library: &LibraryMut, op: F)
        where F: Fn(&mut Document) + 'static + Send {
            library.update(self, op)
        }
    }


    impl From<Link<Document>> for DocumentLink {
        fn from(link: Link<Document>) -> DocumentLink {
            DocumentLink(link)
        }
    }

    impl From<DocumentLink> for Link<Document> {
        fn from(link: DocumentLink) -> Link<Document> {
            link.0
        }
    }

    impl FromYaml<LibraryBuilder> for Marked<DocumentLink> {
        fn from_yaml(
            value: Value,
            context: &LibraryBuilder,
            report: &mut PathReporter
        ) -> Result<Self, Failed> {
            Ok(DocumentLink::build(
                Marked::from_yaml(value, context, report)?,
                context,
                report
            ))
        }
    }
}}

document! (
    ( line, Line, LineLink),
    ( organization, Organization, OrganizationLink),
    ( path, Path, PathLink),
    ( point, Point, PointLink),
    ( source, Source, SourceLink),
    ( structure, Structure, StructureLink),
);
