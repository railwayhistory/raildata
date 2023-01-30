
use std::ops;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{
    DataStore, DocumentLink, FullStore, LinkTarget, StoreLoader,
    XrefsBuilder, XrefsStore,
};
use crate::types::{
    EventDate, Key, IntoMarked, LanguageCode, LanguageText, List, Marked,
    Set, Url,
};
use super::{combined, entity, source};
use super::common::{Common, Progress};


//------------ Link ----------------------------------------------------------

pub use super::combined::SourceLink as Link;


//------------ Document ------------------------------------------------------

pub use super::combined::SourceDocument as Document;

impl<'a> Document<'a> {
    pub fn json(self, _store: &FullStore) -> String {
        self.data().common.json(|json| {
            json.member_str("type", "source");
        })
    }
}


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    link: source::Link,
    pub common: Common,
    pub subtype: Marked<Subtype>,
    
    // Type-dependent attributes
    pub author: List<Marked<entity::Link>>,
    pub collection: Option<Marked<source::Link>>,
    pub date: EventDate,
    pub designation: Option<Marked<String>>,
    pub digital: List<Marked<Url>>,
    pub edition: Option<Marked<String>>,
    pub editor: List<Marked<entity::Link>>,
    pub isbn: Option<Isbn>,
    pub number: Option<Marked<String>>,
    pub organization: List<Marked<entity::Link>>,
    pub pages: Option<Pages>,
    pub publisher: List<Marked<entity::Link>>,
    pub revision: Option<Marked<String>>,
    pub short_title: Option<Marked<String>>,
    pub title: Option<Marked<String>>,
    pub url: Option<Marked<Url>>,
    pub volume: Option<Marked<String>>,

    // Additional attributes
    pub also: List<Marked<source::Link>>,
    pub attribution: Option<Marked<String>>,
    pub crossref: List<Marked<source::Link>>,
    pub note: Option<LanguageText>,
    pub regards: List<Marked<DocumentLink>>,
}

impl Data {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    pub fn link(&self) -> source::Link {
        self.link
    }

    pub fn name(&self, _lang: LanguageCode) -> &str {
        self.key().as_ref()
    }

    pub fn date<'s>(
        &'s self, library: &'s impl LinkTarget<combined::Data>
    ) -> Option<&'s EventDate> {
        if !self.date.is_empty() {
            Some(&self.date)
        }
        else if let Some(collection) = self.collection {
            collection.data(library).date(library)
        }
        else {
            None
        }
    }
}

impl Data {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        link: DocumentLink,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take_default("subtype", context, report);
        let author = doc.take_opt("author", context, report);
        let collection = doc.take_opt("collection", context, report);
        let date = doc.take_default("date", context, report);
        let designation = doc.take_opt("designation", context, report);
        let digital = doc.take_default("digital", context, report);
        let edition = doc.take_opt("edition", context, report);
        let editor = doc.take_default("editor", context, report);
        let isbn = doc.take_opt("isbn", context, report);
        let number = doc.take_opt("number", context, report);
        let organization = doc.take_default("organization", context, report);
        let pages = doc.take_opt("pages", context, report);
        let publisher = doc.take_default("publisher", context, report);
        let revision = doc.take_opt("revision", context, report);
        let short_title = doc.take_opt("short_title", context, report);
        let title = doc.take_opt("title", context, report);
        let url = doc.take_opt("url", context, report);
        let volume = doc.take_opt("volume", context, report);
        let also = doc.take_default("also", context, report);
        let attribution = doc.take_opt("attribution", context, report);
        let crossref = doc.take_default("crossref", context, report);
        let note = doc.take_opt("note", context, report);
        let regards = doc.take_default("regards", context, report);
        doc.exhausted(report)?;
        let source = Data {
            link: link.into(),
            common: common?,
            subtype: subtype?,
            author: author?.into(),
            collection: collection?,
            date: date?,
            designation: designation?,
            digital: digital?,
            edition: edition?,
            editor: editor?,
            isbn: isbn?,
            number: number?,
            organization: organization?,
            pages: pages?,
            publisher: publisher?,
            revision: revision?,
            short_title: short_title?,
            title: title?,
            url: url?,
            volume: volume?,
            also: also?,
            attribution: attribution?,
            crossref: crossref?,
            note: note?,
            regards: regards?,
        };
        source.check_attributes(report)?;
        Ok(source)
    }

    pub fn xrefs(
        &self, 
        builder: &mut XrefsBuilder,
        _store: &crate::store::DataStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        self.author.iter().for_each(|link| {
            link.xrefs_mut(builder).source_author.insert(self.link);
        });
        if let Some(link) = self.collection {
            link.xrefs_mut(builder).source_collection.push(self.link);
        }
        self.editor.iter().for_each(|link| {
            link.xrefs_mut(builder).source_editor.insert(self.link);
        });
        self.organization.iter().for_each(|link| {
            link.xrefs_mut(builder).source_organization.insert(self.link);
        });
        self.publisher.iter().for_each(|link| {
            link.xrefs_mut(builder).source_publisher.insert(self.link);
        });
        self.also.iter().for_each(|link| {
            link.xrefs_mut(builder).source_also.insert(self.link);
        });
        self.crossref.iter().for_each(|link| {
            link.xrefs_mut(builder).source_crossref.insert(self.link);
        });
        self.regards.iter().for_each(|link| {
            link.xrefs_mut(builder).source_regards_mut().insert(self.link);
        });
        Ok(())
    }

    pub fn catalogue(
        &self,
        _builder: &mut CatalogueBuilder,
        _store: &FullStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        Ok(())
    }
}


//------------ Xrefs ---------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Xrefs {
    source_also: Set<Link>,
    source_collection: List<Link>,
    source_crossref: Set<Link>,
    source_regards: Set<Link>,
}

impl Xrefs {
    pub fn source_regards_mut(&mut self) -> &mut Set<Link> {
        &mut self.source_regards
    }

    pub fn finalize(&mut self, store: &DataStore) {
        self.source_collection.sort_by(|left, right| {
            let left = left.data(store);
            let right = right.data(store);
            (left.number.as_ref(), left.volume.as_ref()).cmp(
                &(right.number.as_ref(), right.volume.as_ref())
            )
        });
    }
}


//------------ Meta ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta;

impl Meta {
    pub fn generate(
        _data: &Data, _store: &XrefsStore, _report: &mut PathReporter,
    ) -> Result<Self, Failed> {
        Ok(Meta)
    }
}


//------------ Subtype -------------------------------------------------------

data_enum! {
    pub enum Subtype {
        { Article: "article" }
        { Book: "book" }
        { Inarticle: "inarticle" }
        { Issue: "issue" }
        { Journal: "journal" }
        { Map: "map" }
        { Online: "online" }
        { Series: "series" }
        { Volume: "volume" }
        { Misc: "misc" }
        
        default Misc
    }
}




//------------ Pages ---------------------------------------------------------
//
// XXX Temporary type. Replace with a type encoding the actual specification.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pages(Marked<String>);

impl Pages {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<C> FromYaml<C> for Pages {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        match value.try_into_integer() {
            Ok(int) => {
                Ok(Pages(int.map(|int| format!("{}", int))))
            }
            Err(value) => Marked::from_yaml(value, context, report).map(Pages)
        }
    }
}

impl ops::Deref for Pages {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_value().as_ref()
    }
}


//------------ Isbn ----------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Isbn(Marked<String>);

impl Isbn {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<C> FromYaml<C> for Isbn {
    fn from_yaml(
        value: Value,
        context: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        Marked::from_yaml(value, context, report).map(Isbn)
    }
}

impl ops::Deref for Isbn {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_value().as_ref()
    }
}


//------------ check_attributes ----------------------------------------------

impl Data {
    fn check_attributes(
        &self, report: &mut PathReporter
    ) -> Result<(), Failed> {
        let mut res = Ok(());
        match self.subtype.into_value() {
            Subtype::Article => {
                self.check(
                    self.title.is_some() || self.designation.is_some(),
                    "'title' or 'designation'", report, &mut res
                );
                self.check(
                    self.collection.is_some(), "'collection'",
                    report, &mut res
                );
            }
            Subtype::Book => {
                self.check(
                    !self.author.is_empty() || !self.editor.is_empty()
                        || !self.organization.is_empty(),
                    "'author' or 'editor' or 'organization'",
                    report, &mut res
                );
                self.check(
                    self.title.is_some() || self.designation.is_some(),
                    "'title' or 'designation'", report, &mut res
                );
            }
            Subtype::Inarticle => {
                self.check(
                    self.title.is_some() || self.designation.is_some(),
                    "'title' or 'designation'", report, &mut res
                );
                self.check(
                    self.collection.is_some(), "'collection'",
                    report, &mut res
                );
            }
            Subtype::Issue => {
                self.check(
                    self.collection.is_some(), "'collection'",
                    report, &mut res
                );
                self.check(
                    self.number.is_some(), "'number'", report, &mut res
                );
            }
            Subtype::Journal => {
            }
            Subtype::Map => {
            }
            Subtype::Online => {
            }
            Subtype::Series => {
            }
            Subtype::Volume => {
                self.check(
                    self.collection.is_some(), "'collection'",
                    report, &mut res
                );
                self.check(
                    self.volume.is_some(), "'volume'", report, &mut res
                );
            }
            Subtype::Misc => {
            }
        }
        res
    }

    fn check(
        &self, condition: bool, missing: &'static str,
        report: &mut PathReporter, res: &mut Result<(), Failed>,
    ) {
        if !condition {
            report.error(
                MissingAttribute {
                    subtype: self.subtype.into_value(),
                    missing,
                }.marked(self.origin().location())
            );
            *res = Err(Failed)
        }
    }
}


//============ Errors ========================================================

#[derive(Clone, Debug, Display)]
#[display(
    fmt="missing {} in {} source", missing, subtype
)]
pub struct MissingAttribute {
    subtype: Subtype,
    missing: &'static str,
}

