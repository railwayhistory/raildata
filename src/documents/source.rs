use url::Url;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, Progress, ShortVec, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};
use super::organization::{Organization, OrganizationRef};


//------------ Source --------------------------------------------------------

pub struct Source {
    key: String,
    subtype: Subtype,
    progress: Progress,

    attribution: Option<String>,
    author: Option<ShortVec<String>>,
    collection: Option<SourceRef>,
    crossref: Sources,
    date: Option<Date>,
    designation: Option<String>,
    digital: Option<ShortVec<Url>>,
    edition: Option<String>,
    editor: Option<ShortVec<String>>,
    howpublished: Option<String>,
    institution: Option<OrganizationRef>,
    journal: Option<SourceRef>,
    note: Option<LocalizedString>,
    number: Option<String>,
    organization: Option<OrganizationRef>,
    pages: Option<Pages>,
    publisher: Option<OrganizationRef>,
    regards: ShortVec<DocumentRef>,
    revision: Option<String>,
    series: Option<SourceRef>,
    short_title: Option<String>,
    title: Option<String>,
    volume: Option<String>,
    url: Option<ShortVec<Url>>,
    isbn: Option<String>,
}

impl Source {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn subtype(&self) -> Subtype {
        self.subtype
    }

    pub fn progress(&self) -> Progress {
        self.progress
    }

    pub fn attribution(&self) -> Option<&str> {
        self.attribution.as_ref().map(AsRef::as_ref)
    }

    pub fn author(&self) -> Option<&ShortVec<String>> {
        self.author.as_ref()
    }

    pub fn collection(&self) -> Option<DocumentGuard<Source>> {
        self.collection.as_ref().map(|r| r.get())
    }

    pub fn crossref(&self) -> &Sources {
        &self.crossref
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn designation(&self) -> Option<&str> {
        self.designation.as_ref().map(AsRef::as_ref)
    }

    pub fn digital(&self) -> Option<&ShortVec<Url>> {
        self.digital.as_ref()
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(AsRef::as_ref)
    }

    pub fn editor(&self) -> Option<&ShortVec<String>> {
        self.editor.as_ref()
    }

    pub fn howpublished(&self) -> Option<&str> {
        self.howpublished.as_ref().map(AsRef::as_ref)
    }

    pub fn institution(&self) -> Option<DocumentGuard<Organization>> {
        self.institution.as_ref().map(OrganizationRef::get)
    }

    pub fn journal(&self) -> Option<DocumentGuard<Source>> {
        self.journal.as_ref().map(SourceRef::get)
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn number(&self) -> Option<&str> {
        self.number.as_ref().map(AsRef::as_ref)
    }

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn pages(&self) -> Option<&Pages> {
        self.pages.as_ref()
    }

    pub fn publisher(&self) -> Option<DocumentGuard<Organization>> {
        self.publisher.as_ref().map(OrganizationRef::get)
    }

    pub fn regards(&self) -> &ShortVec<DocumentRef> {
        &self.regards
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_ref().map(AsRef::as_ref)
    }

    pub fn series(&self) -> Option<DocumentGuard<Source>> {
        self.series.as_ref().map(SourceRef::get)
    }

    pub fn short_title(&self) -> Option<&str> {
        self.short_title.as_ref().map(AsRef::as_ref)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> Option<&ShortVec<Url>> {
        self.url.as_ref()
    }

    pub fn volume(&self) -> Option<&str> {
        self.volume.as_ref().map(AsRef::as_ref)
    }

    pub fn isbn(&self) -> Option<&str> {
        self.isbn.as_ref().map(AsRef::as_ref)
    }
}

impl Source {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let subtype = item.parse_default("subtype", builder);
        let progress = item.parse_default("progress", builder);

        let attribution = item.parse_opt("attribution", builder);
        let author = item.parse_opt("author", builder);
        let coll = item.parse_opt("collection", builder);
        let crossref = item.parse_default("crossref", builder);
        let date = item.parse_opt("date", builder);
        let designation = item.parse_opt("designation", builder);
        let digital = item.parse_opt("digital", builder);
        let edition = item.parse_opt("edition", builder);
        let editor = item.parse_opt("editor", builder);
        let howpublished = item.parse_opt("howpublised", builder);
        let institution = item.parse_opt("institution", builder);
        let journal = item.parse_opt("journal", builder);
        let note = item.parse_opt("note", builder);
        let number = item.parse_opt("number", builder);
        let organization = item.parse_opt("organization", builder);
        let pages = item.parse_opt("pages", builder);
        let publisher = item.parse_opt("publisher", builder);
        let regards = item.parse_default("regards", builder);
        let revision = item.parse_opt("revision", builder);
        let series = item.parse_opt("series", builder);
        let short_title = item.parse_opt("short_title", builder);
        let title = item.parse_opt("title", builder);
        let url = item.parse_opt("url", builder);
        let volume = item.parse_opt("volume", builder);
        let isbn = item.parse_opt("isbn", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Document::Source(Source {
            subtype: try_key!(subtype, key),
            progress: try_key!(progress, key),
            attribution: try_key!(attribution, key),
            author: try_key!(author, key),
            collection: try_key!(coll, key),
            crossref: try_key!(crossref, key),
            date: try_key!(date, key),
            designation: try_key!(designation, key),
            digital: try_key!(digital, key),
            edition: try_key!(edition, key),
            editor: try_key!(editor, key),
            howpublished: try_key!(howpublished, key),
            institution: try_key!(institution, key),
            isbn: try_key!(isbn, key),
            journal: try_key!(journal, key),
            note: try_key!(note, key),
            number: try_key!(number, key),
            organization: try_key!(organization, key),
            pages: try_key!(pages, key),
            publisher: try_key!(publisher, key),
            regards: try_key!(regards, key),
            revision: try_key!(revision, key),
            series: try_key!(series, key),
            short_title: try_key!(short_title, key),
            title: try_key!(title, key),
            volume: try_key!(volume, key),
            url: try_key!(url, key),
            key: key
        }))
    }
}


//------------ Subtype -------------------------------------------------------

optional_enum! {
    pub enum Subtype {
        (Article => "article"),
        (Book =>  "book"),
        (Issue => "issue"),
        (Journal => "journal"),
        (Online => "online"),
        (Series => "series"),
        (Volume => "volume"),
        (InArticle => "inarticle"),
        (Misc => "misc"),

        default Misc
    }
}


//------------ Pages ---------------------------------------------------------

/// A range of pages.
///
/// This is either a string of alphanumeric characters for a single page,
/// two such strings connected by a dash for a range of pages, or the
/// literal "insert" followed by white-space and one such string indicating
/// an insert after the given page.
#[derive(Clone, Debug)]
pub enum Pages {
    Single(String),
    Range(String, String),
    Insert(String),
}

impl FromYaml for Pages {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        
        let mut split = item.split_whitespace();
        if let Some("insert") = split.next() {
            let page = match split.next() {
                Some(page) => page,
                None => {
                    builder.error((item.source(), "illegal pages value"));
                    return Err(())
                }
            };
            if let Some(_) = split.next() {
                builder.error((item.source(), "illegal pages value"));
                return Err(())
            }
            return Ok(Pages::Insert(page.into()))
        }

        let mut split = item.splitn(2, '-');
        let lower = match split.next() {
            Some(lower) => lower,
            None => {
                builder.error((item.source(), "illegal pages value"));
                return Err(())
            }
        };
        let upper = split.next();
        if let Some(_) = split.next() {
            builder.error((item.source(), "illegal pages value"));
            return Err(())
        }
        match upper {
            Some(upper) => Ok(Pages::Range(lower.into(), upper.into())),
            None => Ok(Pages::Single(lower.into()))
        }
    }
}


//------------ SourceRef -----------------------------------------------------

pub struct SourceRef(DocumentRef);

impl SourceRef {
    pub fn get(&self) -> DocumentGuard<Source> {
        self.0.get()
    }
}

impl FromYaml for SourceRef {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
        Ok(SourceRef(builder.ref_doc(item.value(), item.source(),
                                     Some(DocumentType::Source))))
    }
}


