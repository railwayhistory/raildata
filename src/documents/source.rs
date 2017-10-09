//! A source document.

use std::ops;
use ::load::construct::{Constructable, Context, Failed};
use ::load::yaml::{MarkedMapping, Value};
use super::common::Common;
use super::links::{DocumentLink, OrganizationLink, SourceLink};
use super::types::{Date, LanguageText, List, Marked, Text, Url};


//------------ Source --------------------------------------------------------

pub struct Source {
    common: Common,
    subtype: Subtype,
    
    // Type-dependent attributes
    author: List<Marked<OrganizationLink>>,
    collection: Option<Marked<SourceLink>>,
    date: Option<Marked<Date>>,
    designation: Option<Text>,
    digital: List<Url>,
    edition: Option<Text>,
    editor: List<Marked<OrganizationLink>>,
    isbn: Option<Isbn>,
    number: Option<Text>,
    organization: List<Marked<OrganizationLink>>,
    pages: Option<Pages>,
    publisher: List<Marked<OrganizationLink>>,
    revision: Option<Text>,
    short_title: Option<Text>,
    title: Option<Text>,
    url: Option<Url>,
    volume: Option<Text>,

    // Additional attributes
    attribution: Option<Text>,
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
    
    pub fn designation(&self) -> Option<&Text> {
        self.designation.as_ref()
    }

    pub fn digital(&self) -> &List<Url> {
        &self.digital
    }

    pub fn edition(&self) -> Option<&Text> {
        self.edition.as_ref()
    }

    pub fn editor(&self) -> &List<Marked<OrganizationLink>> {
        &self.editor
    }

    pub fn isbn(&self) -> Option<&Isbn> {
        self.isbn.as_ref()
    }

    pub fn number(&self) -> Option<&Text> {
        self.number.as_ref()
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

    pub fn revision(&self) -> Option<&Text> {
        self.revision.as_ref()
    }

    pub fn short_title(&self) -> Option<&Text> {
        self.short_title.as_ref()
    }

    pub fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    pub fn volume(&self) -> Option<&Text> {
        self.volume.as_ref()
    }

    pub fn attribution(&self) -> Option<&Text> {
        self.attribution.as_ref()
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
    pub fn construct<C>(common: Common, mut doc: MarkedMapping,
                        context: &mut C) -> Result<Self, Failed>
                     where C: Context {
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
        Ok(Source { common,
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

    pub fn crosslink<C: Context>(&mut self, _context: &mut C) {
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
pub struct Pages(Text);

impl Constructable for Pages {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        match value.try_into_integer() {
            Ok(int) => {
                Ok(Pages(int.map(|int| format!("{}", int))))
            }
            Err(value) => Text::construct(value, context).map(Pages)
        }
    }
}

impl ops::Deref for Pages {
    type Target = Text;

    fn deref(&self) -> &Text {
        &self.0
    }
}


//------------ Isbn ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Isbn(Text);

impl Constructable for Isbn {
    fn construct<C: Context>(value: Value, context: &mut C)
                             -> Result<Self, Failed> {
        Text::construct(value, context).map(Isbn)
    }
}

impl ops::Deref for Isbn {
    type Target = Text;

    fn deref(&self) -> &Text {
        &self.0
    }
}

