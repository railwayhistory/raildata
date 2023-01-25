use derive_more::From;
use paste::paste;
use serde::{Deserialize, Serialize};
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
pub use crate::store::{LinkTarget, DocumentLink, StoreLoader};
use crate::types::{Key, LanguageCode, Location, Marked};
use super::{Line, Entity, Path, Point, Source, Structure};
use super::common::{Common, DocumentType};

macro_rules! document { ( $( ($vattr:ident, $vtype:ident,
                              $vlink:ident ), )* ) => {

    //------------ Data ------------------------------------------------------

    #[derive(Clone, Debug, Deserialize, From, Serialize)]
    pub enum Data {
        $(
            $vtype(super::$vattr::Data),
        )*
    }

    impl Data {
        pub fn common(&self) -> &Common {
            match *self {
                $(
                    Data::$vtype(ref inner) => &inner.common,
                )*
            }
        }

        pub fn common_mut(&mut self) -> &mut Common {
            match *self {
                $(
                    Data::$vtype(ref mut inner) => &mut inner.common,
                )*
            }
        }

        pub fn key(&self) -> &Key {
            &self.common().key
        }

        pub fn doctype(&self) -> DocumentType {
            match *self {
                $(
                    Data::$vtype(_) => DocumentType::$vtype,
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
                    Data::$vtype(ref inner) => inner.name(lang),
                )*
            }
        }

        paste! {
            $(
                pub fn [<try_as_ $vtype:lower>](
                    &self
                ) -> Option<&super::$vattr::Data> {
                    match *self {
                        Data::$vtype(ref inner) => Some(inner),
                        _ => None
                    }
                }
            )*
        }
    }

    impl Data {
        pub fn from_yaml(
            key: Marked<Key>,
            doctype: DocumentType,
            doc: Mapping,
            link: DocumentLink,
            context: &StoreLoader,
            report: &mut PathReporter
        ) -> Result<Self, Failed> {
            match doctype {
                $(
                    DocumentType::$vtype => {
                        $vtype::from_yaml(key, doc, link, context, report)
                            .map(Data::$vtype)
                    }
                )*
            }
        }

        pub fn process_names<F: FnMut(String)>(&self, process: F) {
            match *self {
                $(
                    Data::$vtype(ref inner) => {
                        inner.process_names(process)
                    }
                )*
            }
        }
    }


    //------------ Meta ------------------------------------------------------

    #[derive(Clone, Debug, Deserialize, From, Serialize)]
    pub enum Meta {
        $(
            $vtype(super::$vattr::Meta),
        )*
    }

    impl Meta {
        pub fn generate(
            data: &Data,
            store: &crate::store::StoreEnricher,
            report: &mut crate::load::report::StageReporter,
        ) -> Result<Self, crate::load::report::Failed> {
            match *data {
                $(
                    Data::$vtype(ref inner) => {
                        super::$vattr::Meta::generate(
                            inner, store,
                            &mut report.clone().with_path(
                                inner.origin().path().clone()
                            ),
                        ).map(Meta::$vtype)
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
                store: &StoreLoader,
                report: &mut PathReporter,
            ) -> Marked<Self> {
                store.build_link(key, Some(DocumentType::$vtype), report)
                    .map(Into::into)
            }

            pub fn data(
                self, store: &impl LinkTarget<Data>
            ) -> &super::$vattr::Data {
                match *self.0.data(store) {
                    Data::$vtype(ref inner) => inner,
                    _ => panic!("link to wrong document type")
                }
            }

            pub fn meta(
                self, store: &impl LinkTarget<Meta>
            ) -> &super::$vattr::Meta {
                match *self.0.meta(store) {
                    Meta::$vtype(ref inner) => inner,
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

        impl FromYaml<StoreLoader> for Marked<$vlink> {
            fn from_yaml(
                value: Value,
                context: &StoreLoader,
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
}}

document! (
    ( line, Line, LineLink),
    ( entity, Entity, EntityLink),
    ( path, Path, PathLink),
    ( point, Point, PointLink),
    ( source, Source, SourceLink),
    ( structure, Structure, StructureLink),
);

