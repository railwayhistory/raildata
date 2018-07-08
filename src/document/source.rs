
use std::ops;
use ::load::report::{Failed, Origin, PathReporter, StageReporter};
use ::load::yaml::{FromYaml, Mapping, Value};
use ::types::{Date, Key, LanguageText, List, Marked, Url};
use super::{DocumentLink, OrganizationLink, SourceLink};
use super::common::{Common, Progress};
use super::store::{LoadStore, Stored};


//------------ Source --------------------------------------------------------

#[derive(Clone, Debug)]
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
    also: List<Marked<SourceLink>>,
    attribution: Option<Marked<String>>,
    crossref: List<Marked<SourceLink>>,
    note: Option<LanguageText>,
    regards: List<Marked<DocumentLink>>,
}

impl Source {
    pub fn common(&self) -> &Common {
        &self.common
    }

    pub fn key(&self) -> &Key {
        self.common().key()
    }

    pub fn progress(&self) -> Progress {
        self.common().progress()
    }

    pub fn origin(&self) -> &Origin {
        &self.common().origin()
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype
    }
}

impl<'a> Stored<'a, Source> {
    pub fn author(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.author)
    }

    pub fn collection(&self) -> Option<&Source> {
        self.map_opt(|item| item.collection.as_ref()).map(|x| x.follow())
    }

    pub fn date(&self) -> Option<&Date> {
        self.access().date.as_ref().map(Marked::as_value)
    }

    pub fn designation(&self) -> Option<&str> {
        self.access().designation.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn digital(&self) -> &List<Marked<Url>> {
        &self.access().digital
    }

    pub fn edition(&self) -> Option<&str> {
        self.access().edition.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn editor(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.editor)
    }

    pub fn isbn(&self) -> Option<&Isbn> {
        self.access().isbn.as_ref()
    }

    pub fn number(&self) -> Option<&str> {
        self.access().number.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn organization(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.organization)
    }

    pub fn pages(&self) -> Option<&Pages> {
        self.access().pages.as_ref()
    }

    pub fn publisher(&self) -> Stored<'a, List<Marked<OrganizationLink>>> {
        self.map(|item| &item.publisher)
    }

    pub fn revision(&self) -> Option<&str> {
        self.access().revision.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn short_title(&self) -> Option<&str> {
        self.access().short_title.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn title(&self) -> Option<&str> {
        self.access().title.as_ref().map(|x| x.as_value().as_ref())
    }

    pub fn url(&self) -> Option<&Url> {
        self.access().url.as_ref().map(Marked::as_value)
    }

    pub fn volume(&self) -> Option<&str> {
        self.access().volume.as_ref().map(|x| x.as_value().as_ref())
    }
}

impl Source {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        context: &mut LoadStore,
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
        })
    }

    pub fn verify(&self, _report: &mut StageReporter) {
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

impl<C> FromYaml<C> for Pages {
    fn from_yaml(
        value: Value,
        context: &mut C,
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
        context: &mut C,
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

