
use std::ops;
use std::sync::Arc;
use crate::library::{LibraryBuilder, LibraryMut};
use crate::load::report::{Failed, Origin, PathReporter, StageReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::types::{Date, Key, LanguageText, List, Marked, Set, Url};
use super::{DocumentLink, OrganizationLink, SourceLink};
use super::common::{Common, Progress};

//------------ Source --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Source {
    pub common: Common,
    pub subtype: Subtype,
    
    // Type-dependent attributes
    pub author: List<Marked<OrganizationLink>>,
    pub collection: Option<Marked<SourceLink>>,
    pub date: Option<Marked<Date>>,
    pub designation: Option<Marked<String>>,
    pub digital: List<Marked<Url>>,
    pub edition: Option<Marked<String>>,
    pub editor: List<Marked<OrganizationLink>>,
    pub isbn: Option<Isbn>,
    pub number: Option<Marked<String>>,
    pub organization: List<Marked<OrganizationLink>>,
    pub pages: Option<Pages>,
    pub publisher: List<Marked<OrganizationLink>>,
    pub revision: Option<Marked<String>>,
    pub short_title: Option<Marked<String>>,
    pub title: Option<Marked<String>>,
    pub url: Option<Marked<Url>>,
    pub volume: Option<Marked<String>>,

    // Additional attributes
    pub also: List<Marked<SourceLink>>,
    pub attribution: Option<Marked<String>>,
    pub crossref: List<Marked<SourceLink>>,
    pub note: Option<LanguageText>,
    pub regards: List<Marked<DocumentLink>>,

    // Crosslinks
    pub in_collection: Set<SourceLink>,
    pub variants: Set<SourceLink>, // based on the also field
    pub crossrefed: Set<SourceLink>,
}

impl Source {
    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }
}

impl Source {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &LibraryBuilder,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let subtype = doc.take_default("subtype", context, report);
        let author = doc.take_opt("author", context, report);
        let collection = doc.take_opt("collection", context, report);
        let date = doc.take_opt("date", context, report);
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
        Ok(Source {
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
            in_collection: Set::new(),
            variants: Set::new(),
            crossrefed: Set::new(),
        })
    }

    pub fn crosslink(
        &self,
        link: SourceLink,
        library: &LibraryMut,
        _report: &mut StageReporter
    ) {
        // author
        for target in &self.author {
            target.update(library, move |org| {
                org.source_author.insert(link);
            })
        }

        // collection
        if let Some(target) = self.collection {
            target.update(library, move |source| {
                source.in_collection.insert(link);
            })
        }

        // editor
        for target in &self.editor {
            target.update(library, move |org| {
                org.source_editor.insert(link);
            })
        }

        // organization
        for target in &self.organization {
            target.update(library, move |org| {
                org.source_organization.insert(link);
            })
        }

        // publisher
        for target in &self.publisher {
            target.update(library, move |org| {
                org.source_publisher.insert(link);
            })
        }

        // also -- to make sure we don’t miss any, we add the full also list
        // to all mentioned sources.
        if !self.also.is_empty() {
            let mut also = Vec::with_capacity(self.also.len() + 1);
            also.push(link);
            also.extend(self.also.iter().map(|item| item.into_value()));
            let also = Arc::new(also);
            for target in also.clone().iter() {
                let also = also.clone();
                target.update(library, move |source| {
                    for item in also.iter() {
                        source.variants.insert(*item);
                    }
                })
            }
        }

        // crossref
        for target in &self.crossref {
            target.update(library, move |source| {
                source.crossrefed.insert(link);
            })
        }

        // regards
        for target in &self.regards {
            target.update(library, move |document| {
                document.common_mut().sources.insert(link);
            })
        }
    }

    /*
    pub fn verify(&self, _report: &mut StageReporter) {
    }
    */
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

#[derive(Clone, Debug)]
pub struct Pages(Marked<String>);

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

#[derive(Clone, Debug)]
pub struct Isbn(Marked<String>);

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

