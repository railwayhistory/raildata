use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::types::{Key, Location, Marked};
use ::types::marked::IntoMarked;
use ::document::{Line, Organization, Path, Point, Source, Structure};
use ::document::common::DocumentType;
use super::load::LoadStore;
use super::store::{Store, Stored};
use super::update::UpdateStore;


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

        pub fn crosslink(
            &mut self,
            link: DocumentLink,
            store: &mut UpdateStore,
            report: &mut StageReporter
        ) {
            match *self {
                $(
                    Document::$vtype(ref mut inner) => {
                        inner.crosslink($vlink::from(link), store, report)
                    }
                )*
            }
        }
    }

    $(
        impl AsRef<$vtype> for Document {
            fn as_ref(&self) -> &$vtype {
                match *self {
                    Document::$vtype(ref inner) => inner,
                    _ => panic!("wrong document type"),
                }
            }
        }

        impl<'a> Stored<'a, $vtype> {
            pub fn from_document(doc: Stored<'a, Document>) -> Option<Self> {
                doc.map_opt(|item| {
                    match item {
                        Document::$vtype(ref inner) => Some(inner),
                        _ => None
                    }
                })
            }
        }
    )*


    //------------ StoredDocument --------------------------------------------

    #[derive(Clone, Debug)]
    pub enum StoredDocument<'a> {
        $(
            $vtype(Stored<'a, $vtype>),
        )*
    }

    impl<'a> StoredDocument<'a> {
        pub(crate) fn new(document: &'a Document, store: &'a Store) -> Self {
            match *document {
                $(
                    Document::$vtype(ref inner) => {
                        StoredDocument::$vtype(
                            Stored::new(inner, store)
                        )
                    }
                )*
            }
        }

        pub fn doctype(&self) -> DocumentType {
            match *self {
                $(
                    StoredDocument::$vtype(_) => DocumentType::$vtype,
                )*
            }
        }

        pub fn key(&self) -> &Key {
            match *self {
                $(
                    StoredDocument::$vtype(ref doc) => doc.key(),
                )*
            }
        }

        pub fn origin(&self) -> &Origin {
            match *self {
                $(
                    StoredDocument::$vtype(ref doc) => doc.origin(),
                )*
            }
        }

        pub fn location(&self) -> Location {
            self.origin().location()
        }
    }


    //------------ Links -----------------------------------------------------

    $(
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

            pub fn update<F>(&self, store: &mut UpdateStore, op: F)
            where F: FnOnce(&mut $vtype) {
                store.update(self.pos, |doc| {
                    match doc {
                        Document::$vtype(ref mut inner) => op(inner),
                        _ => panic!("wrong document type"),
                    }
                })
            }

            pub fn from_pos(pos: usize) -> Self {
                $vlink { pos }
            }

            pub fn resolve<'a>(&self, store: &'a Store) -> Stored<'a, $vtype> {
                match store.resolve(self.clone().into()) {
                    StoredDocument::$vtype(inner) => inner,
                    _ => unreachable!()
                }
            }
        }

        impl From<DocumentLink> for $vlink {
            fn from(link: DocumentLink) -> Self {
                $vlink { pos: link.pos }
            }
        }

        impl From<$vlink> for DocumentLink {
            fn from(link: $vlink) -> Self {
                DocumentLink { pos: link.pos }
            }
        }

        impl<'a> Stored<'a, $vlink> {
            pub fn follow(&self) -> Stored<'a, $vtype> {
                match self.store().resolve(DocumentLink::new(self.pos)) {
                    StoredDocument::$vtype(inner) => inner,
                    _ => unreachable!()
                }
            }
        }

        impl<'a> Stored<'a, Marked<$vlink>> {
            pub fn follow(&self) -> Stored<'a, $vtype> {
                self.map(Marked::as_value).follow()
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

    #[derive(Clone, Copy, Debug)]
    pub struct DocumentLink {
        pos: usize
    }

    impl DocumentLink {
        pub fn new(pos: usize) -> Self {
            DocumentLink { pos }
        }

        pub fn pos(&self) -> usize {
            self.pos
        }
    }

    impl<'a> Stored<'a, DocumentLink> {
        pub fn follow(&self) -> StoredDocument<'a> {
            self.store().resolve(self.access().clone())
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

