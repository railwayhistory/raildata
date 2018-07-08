use ::load::report::{Failed, Origin, PathReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::types::{Key, Location, Marked};
use ::types::marked::IntoMarked;
use super::super::{Line, Organization, Path, Point, Source, Structure};
use super::super::common::DocumentType;
use super::load::LoadStore;
use super::store::Stored;


macro_rules! document_enum {  ( $( ($vattr:ident, $vtype:ident,
                                    $vlink:ident ), )* ) => {

    //------------ Document --------------------------------------------------

    #[derive(Clone, Debug)]
    pub enum Document {
        $(
            $vtype($vtype),
        )*
    }

    impl Document {
        pub fn doctype(&self) -> DocumentType {
            match *self {
                $(
                    Document::$vtype(_) => DocumentType::$vtype,
                )*
            }
        }

        pub fn key(&self) -> &Key {
            match *self {
                $(
                    Document::$vtype(ref doc) => doc.key(),
                )*
            }
        }

        pub fn origin(&self) -> &Origin {
            match *self {
                $(
                    Document::$vtype(ref doc) => doc.origin(),
                )*
            }
        }

        pub fn location(&self) -> Location {
            self.origin().location()
        }

        pub fn from_yaml(
            key: Marked<Key>,
            doctype: DocumentType,
            doc: Mapping,
            context: &mut LoadStore,
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
    }

    //------------ Links -----------------------------------------------------

    $(
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $vlink {
            pos: usize
        }

        impl $vlink {
            pub fn forge(
                key: Marked<Key>,
                store: &mut LoadStore,
                report: &mut PathReporter
            ) -> Result<Marked<Self>, Failed> {
                let location = key.location();
                store.forge_link(
                    key, Some(DocumentType::$vtype), report
                ).map(|link| $vlink { pos: link.pos }.marked(location))
            }
        }

        impl<'a> Stored<'a, $vlink> {
            pub fn follow(&self) -> &'a $vtype {
                match self.stored_document(self.access().pos) {
                    Document::$vtype(ref inner) => inner,
                    _ => unreachable!()
                }
            }
        }

        impl<'a> Stored<'a, Marked<$vlink>> {
            pub fn follow(&self) -> &'a $vtype {
                match self.stored_document(self.access().pos) {
                    Document::$vtype(ref inner) => inner,
                    _ => unreachable!()
                }
            }
        }


        impl FromYaml<LoadStore> for Marked<$vlink> {
            fn from_yaml(
                value: Value,
                context: &mut LoadStore,
                report: &mut PathReporter
            ) -> Result<Self, Failed> {
                let location = value.location();
                let key = Marked::from_yaml(value, context, report)?;
                context.forge_link(
                    key, Some(DocumentType::$vtype), report
                ).map(|link| $vlink { pos: link.pos }.marked(location))
            }
        }
    )*


    //------------ DocumentLink ----------------------------------------------

    #[derive(Clone, Debug)]
    pub struct DocumentLink {
        pos: usize
    }

    impl DocumentLink {
        pub fn new(pos: usize) -> Self {
            DocumentLink { pos }
        }
    }

    impl<'a> Stored<'a, DocumentLink> {
        pub fn follow(&self) -> &'a Document {
            self.stored_document(self.access().pos)
        }
    }

    impl FromYaml<LoadStore> for Marked<DocumentLink> {
        fn from_yaml(
            value: Value,
            context: &mut LoadStore,
            report: &mut PathReporter
        ) -> Result<Self, Failed> {
            let key = Marked::from_yaml(value, context, report)?;
            context.forge_link(key, None, report)
        }
    }

}}

document_enum! (
    ( line, Line, LineLink),
    ( organization, Organization, OrganizationLink),
    ( path, Path, PathLink),
    ( point, Point, PointLink),
    ( source, Source, SourceLink),
    ( structure, Structure, StructureLink),
);

