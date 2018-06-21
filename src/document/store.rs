
use std::ops;
use ::load::report::{Failed, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Value};
use ::store::{
    GenericLink, GenericLinkTarget, FindShelf, FillShelf, Link, Shelf,
    StoreBuilder
};
use ::types::{IntoMarked, Key, Marked};
use super::{Line, Organization, Path, Point, Source, Structure};


macro_rules! document_store { ( $( ($vattr:ident, $vtype:ident ), )* ) => {

    //------------ DocumentStore ---------------------------------------------

    #[derive(Clone, Debug)]
    pub struct DocumentStore {
        $(
            $vattr: Shelf<$vtype>,
        )*
        generic: Vec<GenericLinkTarget>,
    }

    impl DocumentStore {
        pub fn new(
            docs: FillDocumentStore,
            generic: Vec<GenericLinkTarget>
        ) -> Self {
            DocumentStore {
                $(
                    $vattr: docs.$vattr.into_shelf().unwrap(),
                )*
                generic
            }
        }

        pub fn len(&self) -> usize {
            0
            $(
                + self.$vattr.len()
            )*
        }
    }


    //------------ FillDocumentStore -----------------------------------------

    #[derive(Clone, Debug, Default)]
    pub struct FillDocumentStore {
        $(
            $vattr: FillShelf<$vtype>,
        )*
    }

    impl FillDocumentStore {
    }

    //------------ FindShelf impls -------------------------------------------

    $(
        impl FindShelf for $vtype {
            type Store = DocumentStore;
            type FillStore = FillDocumentStore;

            fn shelf(store: &Self::Store) -> &Shelf<Self> {
                &store.$vattr
            }

            fn fill_shelf(
                store: &mut Self::FillStore
            ) -> &mut FillShelf<Self> {
                &mut store.$vattr
            }
        }
    )*


    //------------ DocumentStoreBuilder --------------------------------------

    pub type DocumentStoreBuilder = StoreBuilder<FillDocumentStore>;

    impl DocumentStoreBuilder {
        pub fn from_yaml(
            &mut self,
            value: Value,
            report: &mut PathReporter
        ) -> Result<(), Failed> {
            let location = value.location();
            let mut doc = value.into_mapping(report)?;
            let key: Marked<Key> = doc.take("key", self, report)?;
            let doc_type = match doc.take("type", self, report) {
                Ok(doc_type) => doc_type,
                Err(_) => {
                    if let Err(err) = self.insert_broken::<()>(key.into_value(),
                                                         doc.location(),
                                                         report) {
                        report.error(err.marked(location))
                    }
                    return Ok(())
                }
            };
            match doc_type {
                $(
                    DocumentType::$vtype => {
                        match $vtype::from_yaml(key.clone(), doc,
                                                self, report) {
                            Ok(doc) => {
                                if let Err(err) = self.insert(
                                                        key.into_value(),
                                                        doc,
                                                        location, report) {
                                    report.error(err.marked(location))
                                }
                            }
                            Err(_) => {
                                if let Err(err) = self.insert_broken::<$vtype>(
                                                    key.into_value(),
                                                    location, report) {
                                    report.error(err.marked(location))
                                }
                            }
                        }
                    }
                )*
            }
            Ok(())
        }

        pub fn into_store(
            self,
            report: &mut StageReporter
        ) -> Result<DocumentStore, Failed> {
            self.finish(report)
                .map(|(docs, generic)| DocumentStore::new(docs, generic))
        }
    }
            

    //------------ Links -----------------------------------------------------

    $(

        impl FromYaml<DocumentStoreBuilder> for Marked<Link<$vtype>> {
            fn from_yaml(
                value: Value,
                context: &mut DocumentStoreBuilder,
                report: &mut PathReporter
            ) -> Result<Self, Failed> {
                let key = Marked::from_yaml(value, context, report)?;
                context.forge_link(key, report)
            }
        }

        impl FromYaml<DocumentStoreBuilder> for Link<$vtype> {
            fn from_yaml(
                value: Value,
                context: &mut DocumentStoreBuilder,
                report: &mut PathReporter
            ) -> Result<Self, Failed> {
                Marked::from_yaml(value, context, report)
                    .map(Marked::into_value)
            }
        }

    )*
}}

document_store! (
    ( line, Line ),
    ( organization, Organization ),
    ( path, Path ),
    ( point, Point ),
    ( source, Source ),
    ( structure, Structure),
);


//------------ Stored --------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Stored<'a, T: 'a> {
    item: &'a T,
    store: &'a DocumentStore,
}

impl<'a, T: 'a> Stored<'a, T> {
    pub fn access(&self) -> &'a T {
        self.item
    }

    pub fn map<F, U: 'a>(&self, op: F) -> Stored<'a, U>
    where F: FnOnce(&'a T) -> &'a U {
        Stored { item: op(self.item), store: self.store }
    }

    pub fn map_opt<F, U: 'a>(&self, op: F) -> Option<Stored<'a, U>>
    where F: FnOnce(&'a T) -> Option<&'a U> {
        op(self.item).map(|item| {
            Stored { item, store: self.store }
        })
    }

    pub fn wrap<F, U>(&self, op: F) -> ForStored<'a, U>
    where F: FnOnce(&'a T) -> U {
        ForStored { item: op(self.item), store: self.store }
    }
}

impl<'a, T: 'a + FindShelf<Store=DocumentStore>> Stored<'a, Marked<Link<T>>> {
    pub fn follow(&self) -> &'a T {
        self.item.as_value().resolve(self.store)
    }
}


//------------ ForStored -----------------------------------------------------

pub struct ForStored<'a, T> {
    item: T,
    store: &'a DocumentStore,
}

impl<'a, T> ForStored<'a, T> {
    pub fn as_stored<U: 'a>(&self, x: &'a U) -> Stored<'a, U> {
        Stored {
            item: x,
            store: self.store
        }
    }
}

impl<'a, T: 'a> AsRef<T> for ForStored<'a, T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<'a, T: 'a> AsMut<T> for ForStored<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<'a, T: 'a> ops::Deref for ForStored<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item
    }
}


//------------ DocumentLink --------------------------------------------------

pub type DocumentLink = GenericLink;

impl FromYaml<DocumentStoreBuilder> for Marked<DocumentLink> {
    fn from_yaml(
        value: Value,
        context: &mut DocumentStoreBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let location = value.location();
        let key = Marked::from_yaml(value, context, report)?;
        Ok(context.forge_generic(key, report).marked(location))
    }
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

