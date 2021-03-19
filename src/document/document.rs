use derive_more::From;
use paste::paste;
use serde::{Deserialize, Serialize};
use crate::library::{
    LibraryBuilder, LinkTarget, LinkTargetMut
};
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::Link;
use crate::types::{Key, LanguageCode, Location, Marked};
use super::{Line, Organization, Path, Point, Source, Structure};
use super::common::{Common, DocumentType};

macro_rules! document { ( $( ($vattr:ident, $vtype:ident,
                              $vlink:ident ), )* ) => {

    //------------ Document --------------------------------------------------

    #[derive(Clone, Debug, Deserialize, From, Serialize)]
    pub enum Document {
        $(
            $vtype($vtype),
        )*
    }

    impl Document {
        pub fn common(&self) -> &Common {
            match *self {
                $(
                    Document::$vtype(ref inner) => &inner.common,
                )*
            }
        }

        pub fn common_mut(&mut self) -> &mut Common {
            match *self {
                $(
                    Document::$vtype(ref mut inner) => &mut inner.common,
                )*
            }
        }

        pub fn key(&self) -> &Key {
            &self.common().key
        }

        pub fn doctype(&self) -> DocumentType {
            match *self {
                $(
                    Document::$vtype(_) => DocumentType::$vtype,
                )*
            }
        }

        pub fn origin(&self) -> &Origin {
            &self.common().origin
        }

        pub fn location(&self) -> Location {
            self.origin().location()
        }

        pub fn name(&self, lang: LanguageCode) -> &str {
            match *self {
                $(
                    Document::$vtype(ref inner) => inner.name(lang),
                )*
            }
        }

        paste! {
            $(
                pub fn [<try_as_ $vtype:lower>](&self) -> Option<&$vtype> {
                    match *self {
                        Document::$vtype(ref inner) => Some(inner),
                        _ => None
                    }
                }
            )*
        }
    }

    impl Document {
        pub fn from_yaml(
            key: Marked<Key>,
            doctype: DocumentType,
            doc: Mapping,
            link: DocumentLink,
            context: &LibraryBuilder,
            report: &mut PathReporter
        ) -> Result<Self, Failed> {
            match doctype {
                $(
                    DocumentType::$vtype => {
                        $vtype::from_yaml(key, doc, link, context, report)
                            .map(Document::$vtype)
                    }
                )*
            }
        }

        pub fn process_names<F: FnMut(String)>(&self, process: F) {
            match *self {
                $(
                    Document::$vtype(ref inner) => {
                        inner.process_names(process)
                    }
                )*
            }
        }
    }


    //------------ Links -----------------------------------------------------

    $(
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq,
            PartialOrd, Serialize
        )]
        pub struct $vlink(DocumentLink);

        impl $vlink {
            pub fn build(
                key: Marked<Key>,
                store: &LibraryBuilder,
                report: &mut PathReporter,
            ) -> Marked<Self> {
                store.build_link(key, Some(DocumentType::$vtype), report)
                    .map(Into::into)
            }

            pub fn follow(self, library: &impl LinkTarget) -> &$vtype {
                match *library.resolve(self.0) {
                    Document::$vtype(ref inner) => inner,
                    _ => panic!("link to wrong document type")
                }
            }

            pub fn follow_mut(
                self, library: &mut impl LinkTargetMut
            ) -> &mut $vtype {
                match *library.resolve_mut(self.0) {
                    Document::$vtype(ref mut inner) => inner,
                    _ => panic!("link to wrong document type")
                }
            }
        }

        impl From<DocumentLink> for $vlink {
            fn from(link: DocumentLink) -> $vlink {
                $vlink(link)
            }
        }

        impl From<$vlink> for DocumentLink {
            fn from(link: $vlink) -> DocumentLink {
                link.0
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

    #[derive(
        Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
        Serialize
    )]
    pub struct DocumentLink(Link<Document>);

    impl DocumentLink {
        pub fn build(
            key: Marked<Key>,
            store: &LibraryBuilder,
            report: &mut PathReporter,
        ) -> Marked<Self> {
            store.build_link(key, None, report).map(Into::into)
        }

            pub fn follow(self, library: &impl LinkTarget) -> &Document {
                library.resolve(self.0.into())
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

