//! A source document.

use std::ops;
use ::load::construct::{Constructable, ConstructContext, Failed};
use ::load::yaml::{Mapping, Value};
use ::links::{DocumentLink, OrganizationLink, SourceLink};
use ::types::{Date, LanguageText, Key, List, Marked, Url};
use super::common::Common;


//------------ Source --------------------------------------------------------

pub struct Source {
    common: Common,
    subtype: Subtype,
    
    // Type-dependent attributes
    author: List<Marked<OrganizationLink>>,
    collection: Option<Marked<SourceLink>>,
    date: Option<Marked<Date>>,
    designation: Option<Marked<String>>,
    digital: List<Marked<Url>>,
    edition: Option<Marked<String>>,
    editor: List<Marked<OrganizationLink>>,
    isbn: Option<Isbn>,
    number: Option<Marked<String>>,
    organization: List<Marked<OrganizationLink>>,
    pages: Option<Pages>,
    publisher: List<Marked<OrganizationLink>>,
    revision: Option<Marked<String>>,
    short_title: Option<Marked<String>>,
    title: Option<Marked<String>>,
    url: Option<Marked<Url>>,
    volume: Option<Marked<String>>,

    // Additional attributes
    attribution: Option<Marked<String>>,
    crossref: List<Marked<SourceLink>>,
    note: Option<LanguageText>,
    regards: List<Marked<DocumentLink>>,
}

impl Source {
    pub fn subtype(&self) -> Subtype {
        self.subtype
    }
    
    pub fn author(&self) -> &List<Marked<OrganizationLink>> {
        &self.author
    }

    pub fn collection(&self) -> Option<&Marked<SourceLink>> {
        self.collection.as_ref()
    }

    pub fn date(&self) -> Option<&Marked<Date>> {
        self.date.as_ref()
    }
    
    pub fn designation(&self) -> Option<&str> {
        self.designation.as_ref().map(AsRef::as_ref)
    }

    pub fn digital(&self) -> &List<Marked<Url>> {
        &self.digital
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(AsRef::as_ref)
    }

    pub fn editor(&self) -> &List<Marked<OrganizationLink>> {
        &self.editor
    }

    pub fn isbn(&self) -> Option<&Isbn> {
        self.isbn.as_ref()
    }

    pub fn number(&self) -> Option<&str> {
        self.number.as_ref().map(AsRef::as_ref)
    }

    pub fn organization(&self) -> &List<Marked<OrganizationLink>> {
        &self.organization
    }

    pub fn pages(&self) -> Option<&Pages> {
        self.pages.as_ref()
    }

    pub fn publisher(&self) -> &List<Marked<OrganizationLink>> {
        &self.publisher
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_ref().map(AsRef::as_ref)
    }

    pub fn short_title(&self) -> Option<&str> {
        self.short_title.as_ref().map(AsRef::as_ref)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref().map(Marked::as_value)
    }

    pub fn volume(&self) -> Option<&str> {
        self.volume.as_ref().map(AsRef::as_ref)
    }

    pub fn attribution(&self) -> Option<&str> {
        self.attribution.as_ref().map(AsRef::as_ref)
    }

    pub fn crossref(&self) -> &List<Marked<SourceLink>> {
        &self.crossref
    }

    pub fn note(&self) -> Option<&LanguageText> {
        self.note.as_ref()
    }

    pub fn regards(&self) -> &List<Marked<DocumentLink>> {
        &self.regards
    }
}

impl Source {
    pub fn construct(key: Marked<Key>, mut doc: Marked<Mapping>,
                     context: &mut ConstructContext) -> Result<Self, Failed> {
        let common = Common::construct(key, &mut doc, context);
        let subtype = doc.take_default("subtype", context);
        let author = doc.take_opt("author", context);
        let collection = doc.take_opt("collection", context);
        let date = doc.take_opt("date", context);
        let designation = doc.take_opt("designation", context);
        let digital = doc.take_default("digital", context);
        let edition = doc.take_opt("edition", context);
        let editor = doc.take_default("editor", context);
        let isbn = doc.take_opt("isbn", context);
        let number = doc.take_opt("number", context);
        let organization = doc.take_default("organization", context);
        let pages = doc.take_opt("pages", context);
        let publisher = doc.take_default("publisher", context);
        let revision = doc.take_opt("revision", context);
        let short_title = doc.take_opt("short_title", context);
        let title = doc.take_opt("title", context);
        let url = doc.take_opt("url", context);
        let volume = doc.take_opt("volume", context);
        let attribution = doc.take_opt("attribution", context);
        let crossref = doc.take_default("crossref", context);
        let note = doc.take_opt("note", context);
        let regards = doc.take_default("regards", context);
        doc.exhausted(context)?;
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
            attribution: attribution?,
            crossref: crossref?,
            note: note?,
            regards: regards?,
        })
    }
}


impl ops::Deref for Source {
    type Target = Common;

    fn deref(&self) -> &Common {
        &self.common
    }
}

impl ops::DerefMut for Source {
    fn deref_mut(&mut self) -> &mut Common {
        &mut self.common
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

#[derive(Clone, Debug)]
pub struct Pages(Marked<String>);

impl Constructable for Pages {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        match value.try_into_integer() {
            Ok(int) => {
                Ok(Pages(int.map(|int| format!("{}", int))))
            }
            Err(value) => Marked::construct(value, context).map(Pages)
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

impl Constructable for Isbn {
    fn construct(value: Value, context: &mut ConstructContext)
                 -> Result<Self, Failed> {
        Marked::construct(value, context).map(Isbn)
    }
}

impl ops::Deref for Isbn {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_value().as_ref()
    }
}

