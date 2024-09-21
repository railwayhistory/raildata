use derive_more::From;
use paste::paste;
use serde::{Deserialize, Serialize};
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
pub use crate::store::{
    DataStore, FullStore, LinkTarget, LinkTargetMut, DocumentLink,
    StoreLoader, XrefsBuilder,
};
use crate::types::{Key, Location, Marked, Set};
use super::source;
use super::common::{Common, DocumentType};

pub use crate::store::DocumentLink as Link;

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
                        super::$vattr::Data::from_yaml(
                            key, doc, link, context, report
                        ).map(Data::$vtype)
                    }
                )*
            }
        }

        pub fn default_xrefs(&self) -> Xrefs {
            match *self {
                $(
                    Data::$vtype(_) => {
                        Xrefs::$vtype(super::$vattr::Xrefs::default())
                    }
                )*
            }
        }

        pub fn xrefs(
            &self,
            builder: &mut XrefsBuilder,
            store: &crate::store::DataStore,
            report: &mut crate::load::report::StageReporter,
        ) -> Result<(), crate::load::report::Failed> {
            match *self {
                $(
                    Data::$vtype(ref inner) => {
                        inner.xrefs(
                            builder, store,
                            &mut report.clone().with_path(
                                inner.origin().path().clone()
                            ),
                        )
                    }
                )*
            }
        }

        pub fn catalogue(
            &self,
            builder: &mut CatalogueBuilder,
            store: &FullStore,
            report: &StageReporter,
        ) -> Result<(), Failed> {
            match *self {
                $(
                    Data::$vtype(ref inner) => {
                        inner.catalogue(
                            builder, store,
                            &mut report.clone().with_path(
                                inner.origin().path().clone()
                            ),
                        )
                    }
                )*
            }
        }
    }


    //------------ Xrefs -----------------------------------------------------

    #[derive(Clone, Debug, Deserialize, From, Serialize)]
    pub enum Xrefs {
        $(
            $vtype(super::$vattr::Xrefs),
        )*
    }

    impl Xrefs {
        pub fn source_regards_mut(&mut self) -> &mut Set<source::Link> {
            match *self {
                $(
                    Xrefs::$vtype(ref mut inner) => {
                        inner.source_regards_mut()
                    }
                )*
            }
        }

        pub fn finalize(&mut self, store: &DataStore) {
            match *self {
                $(
                    Xrefs::$vtype(ref mut inner) => inner.finalize(store),
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
            store: &crate::store::XrefsStore,
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

            pub fn xrefs(
                self, store: &impl LinkTarget<Xrefs>
            ) -> &super::$vattr::Xrefs {
                match *self.0.xrefs(store) {
                    Xrefs::$vtype(ref inner) => inner,
                    _ => panic!("link to wrong document type")
                }
            }

            pub fn xrefs_mut(
                self, store: &mut impl LinkTargetMut<Xrefs>
            ) -> &mut super::$vattr::Xrefs {
                match *self.0.xrefs_mut(store) {
                    Xrefs::$vtype(ref mut inner) => inner,
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

            pub fn document(
                self, store: &FullStore,
            ) -> super::$vattr::Document {
                match self.0.document(store) {
                    Document::$vtype(inner) => inner,
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

    //------------ Documents -------------------------------------------------

    paste! {
        #[derive(Clone, Copy, Debug)]
        pub enum Document<'a> {
            $(
                $vtype([<$vtype Document>]<'a>),
            )*
        }

        impl<'a> Document<'a> {
            pub fn new(
                data: &'a Data, xrefs: &'a Xrefs, meta: &'a Meta
            ) -> Self {
                match *data {
                    $(
                        Data::$vtype(ref data) => {
                            let xrefs = match *xrefs {
                                Xrefs::$vtype(ref xrefs) => xrefs,
                                _ => panic!("document type mismatch"),
                            };
                            let meta = match *meta {
                                Meta::$vtype(ref meta) => meta,
                                _ => panic!("document type mismatch"),
                            };
                            Document::$vtype(
                                [<$vtype Document>]::new(data, xrefs, meta)
                            )
                        }
                    )*
                }
            }

            pub fn key(self) -> &'a Key {
                match self {
                    $(
                        Document::$vtype(doc) => doc.key(),
                    )*
                }
            }

            $(
                paste! {
                    pub fn [< try_as_ $vtype:lower >](
                        self
                    ) -> Option<[< $vtype Document >]<'a>> {
                        match self {
                            Document::$vtype(inner) => Some(inner),
                            _ => None
                        }
                    }
                }
            )*
        }
    }

    $(
        paste! {
            #[derive(Clone, Copy, Debug)]
            pub struct [<$vtype Document>] <'a> {
                data: &'a super::$vattr::Data,
                xrefs: &'a super::$vattr::Xrefs,
                meta: &'a super::$vattr::Meta,
            }

            impl<'a> [<$vtype Document>] <'a> {
                pub fn new(
                    data: &'a super::$vattr::Data,
                    xrefs: &'a super::$vattr::Xrefs,
                    meta: &'a super::$vattr::Meta,
                ) -> Self {
                    [<$vtype Document>] { data, xrefs, meta }
                }

                pub fn key(self) -> &'a Key {
                    self.data().key()
                }

                pub fn doctype(self) -> DocumentType {
                    DocumentType::$vtype
                }

                pub fn data(self) -> &'a super::$vattr::Data {
                    self.data
                }

                pub fn xrefs(self) -> &'a super::$vattr::Xrefs {
                    self.xrefs
                }

                pub fn meta(self) -> &'a super::$vattr::Meta {
                    self.meta
                }
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

