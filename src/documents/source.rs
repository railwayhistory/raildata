use std::str::FromStr;
use url::Url;
use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::yaml::{FromYaml, Item, Mapping, ValueItem};
use super::common::{LocalizedString, ShortVec, Sources};
use super::date::Date;
use super::document::{Document, DocumentType};
use super::organization::{Organization, OrganizationRef};


//------------ Source --------------------------------------------------------

pub enum Source {
    Article(Article),
    Book(Book),
    Issue(Issue),
    Journal(Journal),
    Online(Online),
    Misc(Misc),
}

impl Source {
    pub fn key(&self) -> &str {
        match *self {
            Source::Article(ref doc) => doc.key(),
            Source::Book(ref doc) => doc.key(),
            Source::Issue(ref doc) => doc.key(),
            Source::Journal(ref doc) => doc.key(),
            Source::Online(ref doc) => doc.key(),
            Source::Misc(ref doc) => doc.key(),
        }
    }
}

impl Source {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     builder: &CollectionBuilder)
                     -> Result<Document, Option<String>> {
        let subtype = try_key!(item.parse_default("subtype", builder), key);
        Ok(Document::Source(match subtype {
            Subtype::Article => {
                Source::Article(Article::from_yaml(key, item, builder)?)
            }
            Subtype::Book => {
                Source::Book(Book::from_yaml(key, item, builder)?)
            }
            Subtype::Issue => {
                Source::Issue(Issue::from_yaml(key, item, builder)?)
            }
            Subtype::Journal => {
                Source::Journal(Journal::from_yaml(key, item, builder)?)
            }
            Subtype::Online => {
                Source::Online(Online::from_yaml(key, item, builder)?)
            }
            Subtype::Misc => {
                Source::Misc(Misc::from_yaml(key, item, builder)?)
            }
        }))
    }
}


//------------ Article -------------------------------------------------------

pub struct Article {
    key: String,
    author: Option<ShortVec<String>>,
    collection: Option<SourceRef>,
    crossref: Sources,
    date: Option<Date>,
    editor: Option<ShortVec<String>>,
    note: Option<LocalizedString>,
    pages: Option<Pages>,
    regards: ShortVec<DocumentRef>,
    revision: Option<String>,
    title: String,
    url: Option<ShortVec<Url>>,
}

impl Article {
    pub fn key(&self) -> &str {
        &self.key
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

    pub fn editor(&self) -> Option<&ShortVec<String>> {
        self.editor.as_ref()
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn pages(&self) -> Option<Pages> {
        self.pages
    }

    pub fn regards(&self) -> &ShortVec<DocumentRef> {
        &self.regards
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_ref().map(AsRef::as_ref)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> Option<&ShortVec<Url>> {
        self.url.as_ref()
    }
}

impl Article {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let author = item.parse_opt("author", builder);
        let coll = item.parse_opt("collection", builder);
        let crossref = item.parse_default("crossref", builder);
        let date = item.parse_opt("date", builder);
        let editor = item.parse_opt("editor", builder);
        let note = item.parse_opt("note", builder);
        let pages = item.parse_opt("pages", builder);
        let regards = item.parse_default("regards", builder);
        let revision = item.parse_opt("revision", builder);
        let title = item.parse_mandatory("title", builder);
        let url = item.parse_opt("url", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Article {
            author: try_key!(author, key),
            collection: try_key!(coll, key),
            crossref: try_key!(crossref, key),
            date: try_key!(date, key),
            editor: try_key!(editor, key),
            note: try_key!(note, key),
            pages: try_key!(pages, key),
            regards: try_key!(regards, key),
            revision: try_key!(revision, key),
            title: try_key!(title, key),
            url: try_key!(url, key),
            key: key,
        })
    }
}


//------------ Book ----------------------------------------------------------

pub struct Book {
    key: String,
    author: Option<ShortVec<String>>,
    date: Option<Date>,
    edition: Option<String>,
    editor: Option<ShortVec<String>>,
    howpublished: Option<String>,
    institution: Option<OrganizationRef>,
    note: Option<LocalizedString>,
    organization: Option<OrganizationRef>,
    publisher: Option<OrganizationRef>,
    series: Option<SourceRef>,
    title: String,
    volume: Option<String>,
    isbn: Option<String>,
}

impl Book {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn author(&self) -> Option<&ShortVec<String>> {
        self.author.as_ref()
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
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

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn publisher(&self) -> Option<DocumentGuard<Organization>> {
        self.publisher.as_ref().map(OrganizationRef::get)
    }

    pub fn series(&self) -> Option<DocumentGuard<Source>> {
        self.series.as_ref().map(SourceRef::get)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn volume(&self) -> Option<&str> {
        self.volume.as_ref().map(AsRef::as_ref)
    }

    pub fn isbn(&self) -> Option<&str> {
        self.isbn.as_ref().map(AsRef::as_ref)
    }
}

impl Book {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let author = item.parse_opt("author", builder);
        let date = item.parse_opt("date", builder);
        let edition = item.parse_opt("edition", builder);
        let editor = item.parse_opt("editor", builder);
        let howpublished = item.parse_opt("howpublised", builder);
        let institution = item.parse_opt("institution", builder);
        let note = item.parse_opt("note", builder);
        let organization = item.parse_opt("organization", builder);
        let publisher = item.parse_opt("publisher", builder);
        let series = item.parse_opt("series", builder);
        let title = item.mandatory_key("title", builder)
                        .and_then(|item| item.into_string(builder));
        let volume = item.parse_opt("volume", builder);
        let isbn = item.parse_opt("isbn", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Book {
            author: try_key!(author, key),
            date: try_key!(date, key),
            edition: try_key!(edition, key),
            editor: try_key!(editor, key),
            howpublished: try_key!(howpublished, key),
            institution: try_key!(institution, key),
            note: try_key!(note, key),
            organization: try_key!(organization, key),
            publisher: try_key!(publisher, key),
            series: try_key!(series, key),
            title: try_key!(title, key),
            volume: try_key!(volume, key),
            isbn: try_key!(isbn, key),
            key: key,
        })
    }
}


//------------ Issue ---------------------------------------------------------

pub struct Issue {
    key: String,
    date: Option<Date>,
    editor: Option<String>,
    institution: Option<OrganizationRef>,
    journal: Option<SourceRef>,
    number: Option<String>,
    organization: Option<OrganizationRef>,
    publisher: Option<OrganizationRef>,
    title: Option<String>,
    volume: Option<String>,
    url: Option<ShortVec<Url>>,
    short_title: Option<String>,
}

impl Issue {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn editor(&self) -> Option<&str> {
        self.editor.as_ref().map(AsRef::as_ref)
    }

    pub fn institution(&self) -> Option<DocumentGuard<Organization>> {
        self.institution.as_ref().map(OrganizationRef::get)
    }

    pub fn journal(&self) -> Option<DocumentGuard<Source>> {
        self.journal.as_ref().map(SourceRef::get)
    }

    pub fn number(&self) -> Option<&str> {
        self.number.as_ref().map(AsRef::as_ref)
    }

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn publisher(&self) -> Option<DocumentGuard<Organization>> {
        self.publisher.as_ref().map(OrganizationRef::get)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn volume(&self) -> Option<&str> {
        self.volume.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> Option<&ShortVec<Url>> {
        self.url.as_ref()
    }

    pub fn short_title(&self) -> Option<&str> {
        self.short_title.as_ref().map(AsRef::as_ref)
    }
}

impl Issue {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let date = item.parse_opt("date", builder);
        let editor = item.parse_opt("editor", builder);
        let institution = item.parse_opt("institution", builder);
        let journal = item.parse_opt("journal", builder);
        let number = item.parse_opt("number", builder);
        let organization = item.parse_opt("organization", builder);
        let publisher = item.parse_opt("publisher", builder);
        let title = item.parse_opt("title", builder);
        let volume = item.parse_opt("volume", builder);
        let url = item.parse_opt("url", builder);
        let short_title = item.parse_opt("short_title", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Issue {
            date: try_key!(date, key),
            editor: try_key!(editor, key),
            institution: try_key!(institution, key),
            journal: try_key!(journal, key),
            number: try_key!(number, key),
            organization: try_key!(organization, key),
            publisher: try_key!(publisher, key),
            title: try_key!(title, key),
            volume: try_key!(volume, key),
            url: try_key!(url, key),
            short_title: try_key!(short_title, key),
            key: key,
        })
    }
}


//------------ Journal -------------------------------------------------------

pub struct Journal {
    key: String,
    author: Option<ShortVec<String>>,
    date: Option<Date>,
    editor: Option<ShortVec<String>>,
    howpublished: Option<String>,
    institution: Option<OrganizationRef>,
    organization: Option<OrganizationRef>,
    publisher: Option<OrganizationRef>,
    title: Option<String>,
    url: Option<ShortVec<Url>>,
    short_title: Option<String>,
}

impl Journal {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn author(&self) -> Option<&ShortVec<String>> {
        self.author.as_ref()
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
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

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn publisher(&self) -> Option<DocumentGuard<Organization>> {
        self.publisher.as_ref().map(OrganizationRef::get)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> Option<&ShortVec<Url>> {
        self.url.as_ref()
    }

    pub fn short_title(&self) -> Option<&str> {
        self.short_title.as_ref().map(AsRef::as_ref)
    }
}

impl Journal {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let author = item.parse_opt("author", builder);
        let date = item.parse_opt("date", builder);
        let editor = item.parse_opt("editor", builder);
        let howpublished = item.parse_opt("howpublished", builder);
        let institution = item.parse_opt("institution", builder);
        let organization = item.parse_opt("organization", builder);
        let publisher = item.parse_opt("publisher", builder);
        let title = item.parse_opt("title", builder);
        let url = item.parse_opt("url", builder);
        let short_title = item.parse_opt("short_title", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Journal {
            author: try_key!(author, key),
            date: try_key!(date, key),
            editor: try_key!(editor, key),
            howpublished: try_key!(howpublished, key),
            institution: try_key!(institution, key),
            organization: try_key!(organization, key),
            publisher: try_key!(publisher, key),
            title: try_key!(title, key),
            url: try_key!(url, key),
            short_title: try_key!(short_title, key),
            key: key,
        })
    }
}


//------------ Online --------------------------------------------------------

pub struct Online {
    key: String,
    author: Option<ShortVec<String>>,
    date: Option<Date>,
    edition: Option<String>,
    editor: Option<ShortVec<String>>,
    institution: Option<OrganizationRef>,
    organization: Option<OrganizationRef>,
    title: Option<String>,
    url: Url,
    short_title: Option<String>,
}

impl Online {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn author(&self) -> Option<&ShortVec<String>> {
        self.author.as_ref()
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn editor(&self) -> Option<&ShortVec<String>> {
        self.editor.as_ref()
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(AsRef::as_ref)
    }

    pub fn institution(&self) -> Option<DocumentGuard<Organization>> {
        self.institution.as_ref().map(OrganizationRef::get)
    }

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn short_title(&self) -> Option<&str> {
        self.short_title.as_ref().map(AsRef::as_ref)
    }
}

impl Online {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let author = item.parse_opt("author", builder);
        let date = item.parse_opt("date", builder);
        let edition = item.parse_opt("edition", builder);
        let editor = item.parse_opt("editor", builder);
        let institution = item.parse_opt("institution", builder);
        let organization = item.parse_opt("organization", builder);
        let title = item.parse_opt("title", builder);
        let url = item.parse_mandatory("url", builder);
        let short_title = item.parse_opt("short_title", builder);
        try_key!(item.exhausted(builder), key);

        Ok(Online {
            author: try_key!(author, key),
            date: try_key!(date, key),
            edition: try_key!(edition, key),
            editor: try_key!(editor, key),
            institution: try_key!(institution, key),
            organization: try_key!(organization, key),
            title: try_key!(title, key),
            url: try_key!(url, key),
            short_title: try_key!(short_title, key),
            key: key,
        })
    }
}


//------------ Misc ----------------------------------------------------------

pub struct Misc {
    key: String,
    author: Option<ShortVec<String>>,
    date: Option<Date>,
    editor: Option<ShortVec<String>>,
    edition: Option<String>,
    institution: Option<OrganizationRef>,
    note: Option<LocalizedString>,
    organization: Option<OrganizationRef>,
    series: Option<String>,
    title: Option<String>,
    url: Option<ShortVec<Url>>,
}

impl Misc {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn author(&self) -> Option<&ShortVec<String>> {
        self.author.as_ref()
    }

    pub fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    pub fn edition(&self) -> Option<&str> {
        self.edition.as_ref().map(AsRef::as_ref)
    }

    pub fn editor(&self) -> Option<&ShortVec<String>> {
        self.editor.as_ref()
    }

    pub fn institution(&self) -> Option<DocumentGuard<Organization>> {
        self.institution.as_ref().map(OrganizationRef::get)
    }

    pub fn note(&self) -> Option<&LocalizedString> {
        self.note.as_ref()
    }

    pub fn organization(&self) -> Option<DocumentGuard<Organization>> {
        self.organization.as_ref().map(OrganizationRef::get)
    }

    pub fn series(&self) -> Option<&str> {
        self.series.as_ref().map(AsRef::as_ref)
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(AsRef::as_ref)
    }

    pub fn url(&self) -> Option<&ShortVec<Url>> {
        self.url.as_ref()
    }
}

impl Misc {
    fn from_yaml(key: String, mut item: Item<Mapping>,
                 builder: &CollectionBuilder) -> Result<Self, Option<String>> {
        let author = item.parse_opt("author", builder);
        let date = item.parse_opt("date", builder);
        let edition = item.parse_opt("edition", builder);
        let editor = item.parse_opt("editor", builder);
        let note = item.parse_opt("note", builder);
        let institution = item.parse_opt("institution", builder);
        let organization = item.parse_opt("organization", builder);
        let series = item.parse_opt("series", builder);
        let title = item.parse_opt("title", builder);
        let url = item.parse_opt("url", builder);
        try_key!(item.exhausted(builder), key);
        Ok(Misc {
            author: try_key!(author, key),
            date: try_key!(date, key),
            edition: try_key!(edition, key),
            editor: try_key!(editor, key),
            institution: try_key!(institution, key),
            note: try_key!(note, key),
            organization: try_key!(organization, key),
            series: try_key!(series, key),
            title: try_key!(title, key),
            url: try_key!(url, key),
            key: key,
        })
    }
}


//------------ Subtype -------------------------------------------------------

optional_enum! {
    pub enum Subtype {
        (Article => "article"),
        (Book => "book"),
        (Issue => "issue"),
        (Journal => "journal"),
        (Online => "online"),
        (Misc => "misc"),

        default Misc
    }
}


//------------ Pages ---------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Pages {
    lower: i64,
    upper: Option<i64>,
}

impl FromYaml for Pages {
    fn from_yaml(item: ValueItem, builder: &CollectionBuilder)
                 -> Result<Self, ()> {
        let item = item.into_string_item(builder)?;
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
        let lower = i64::from_str(lower).map_err(|_| {
            builder.error((item.source(), "illegal pages value"));
        })?;
        let upper = match upper {
            Some(upper) => {
                Some(i64::from_str(upper).map_err(|_| {
                    builder.error((item.source(), "illegal pages value"));
                })?)
            },
            None => None
        };
        Ok(Pages{lower: lower, upper: upper})
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


