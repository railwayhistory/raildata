use std::collections::HashMap;
use std::str::FromStr;
use radix_trie::{Trie, TrieCommon};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use crate::document::{
    Document, DocumentLink, Line, LineLink, Organization, OrganizationLink
};
use crate::library::Library;
use crate::types::{CountryCode, Key, List};


//------------ Catalogue -----------------------------------------------------

/// The meta-information we keep for finding documents.
#[derive(Clone, Default)]
pub struct Catalogue {
    /// Index of document names.
    ///
    /// This is the primary search index.
    names: Trie<String, List<(String, DocumentLink)>>,

    /// Index of countries in alphabetical order of country code.
    countries: Vec<OrganizationLink>,

    /// The number of documents for the various types.
    doc_nums: DocumentNumbers,

    /// Index of lines for countries in order of line code.
    country_lines: CountryLines,
}

impl Catalogue {
    pub fn new(library: &Library) -> Self {
        let mut res = Self::default();
        library.links().for_each(|link| res.process_link(link, library));
        res.finalize(library);
        res
    }

    fn process_link(&mut self, link: DocumentLink, library: &Library) {
        let document = library.resolve(link);

        self.count_doc(document);
        self.insert_names(link, document);

        match *document {
            Document::Line(ref inner) => {
                self.process_line(link.into(), inner, library)
            }
            Document::Organization(ref inner) => {
                self.process_organization(link.into(), inner, library)
            }
            _ => { }
        }
    }

    fn count_doc(&mut self, doc: &Document) {
        self.doc_nums.total += 1;
        match *doc {
            Document::Line(_) => self.doc_nums.lines += 1,
            Document::Organization(_) => self.doc_nums.organizations += 1,
            Document::Path(_) => self.doc_nums.paths += 1,
            Document::Point(_) => self.doc_nums.points += 1,
            Document::Source(_) => self.doc_nums.sources += 1,
            Document::Structure(_) => self.doc_nums.structures += 1,
        }
    }

    fn insert_names(
        &mut self,
        link: DocumentLink,
        document: &Document
    ) {
        self.insert_name(document.key().to_string(), link);
        document.process_names(|name| self.insert_name(name, link))
    }

    fn insert_name(&mut self, name: String, link: DocumentLink) {
        let term = Self::normalize_name(&name);
        if let Some(value) = self.names.get_mut(&term) {
            value.push((name, link))
        }
        else {
            self.names.insert(term, List::with_value((name, link)));
        }
    }

    fn normalize_name(name: &str) -> String {
        name.nfd()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect()
    }

    fn process_line(
        &mut self,
        link: LineLink,
        line: &Line,
        library: &Library
    ) {
        self.country_lines.add_line(link, line, library);
    }

    fn process_organization(
        &mut self,
        link: OrganizationLink,
        organization: &Organization,
        _library: &Library
    ) {
        if organization.subtype.is_country() {
            self.countries.push(link)
        }
    }

    fn finalize(&mut self, library: &Library) {
        self.countries.sort_unstable_by(|left, right| {
            left.follow(library).key().cmp(
                right.follow(library).key()
            )
        });
        self.country_lines.finalize(library);
    }
}

impl Catalogue {
    pub fn search_names(
        &self, prefix: &str, count: usize
    ) -> impl Iterator<Item = (&str, DocumentLink)> {
        let prefix = Self::normalize_name(prefix);
        self.names.get_raw_ancestor(&prefix).iter()
            .filter(move |(key, _)| key.starts_with(&prefix))
            .flat_map(|(_, value)| value)
            .map(|(name, link)| (name.as_str(), *link))
            .take(count)
    }

    pub fn countries<'a>(
        &'a self
    ) -> impl Iterator<Item = OrganizationLink> + 'a {
        self.countries.iter().cloned()
    }

    pub fn doc_nums(
        &self
    ) -> DocumentNumbers {
        self.doc_nums
    }

    pub fn country_lines(&self) -> &CountryLines {
        &self.country_lines
    }
}


//------------ DocumentNumbers -----------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct DocumentNumbers {
    pub total: usize,
    pub lines: usize,
    pub organizations: usize,
    pub paths: usize,
    pub points: usize,
    pub sources: usize,
    pub structures: usize,
}


//------------ CountryLines --------------------------------------------------

/// An index of lines for each country.
///
/// The index is based on the country code portion of the line code. For each
/// country, the lines are ordered by their code.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CountryLines {
    countries: HashMap<CountryCode, Option<OrganizationLink>>,
    index: HashMap<OrganizationLink, Vec<LineLink>>,
}

impl CountryLines {
    fn add_line(&mut self, link: LineLink, line: &Line, library: &Library) {
        let code = match line.jurisdiction() {
            Some(code) => code,
            None => return,
        };

        let org = match self.countries.entry(code).or_insert_with(|| {
            let key = match Key::from_string(format!("org.{}", code)) {
                Ok(key) => key,
                Err(_) => return None
            };
            library.get(&key).map(Into::into)
        }) {
            Some(org) => *org,
            None => return
        };
        self.index.entry(org).or_default().push(link);
    }

    fn finalize(&mut self, library: &Library) {
        for lines in self.index.values_mut() {
            lines.sort_unstable_by(|left, right| {
                left.follow(library).key().cmp(
                    right.follow(library).key()
                )
            })
        }
    }

    pub fn by_link(&self, link: OrganizationLink) -> &[LineLink] {
        self.index.get(&link).map(Vec::as_slice).unwrap_or_default()
    }

    pub fn by_code(&self, code: CountryCode) -> &[LineLink] {
        match self.countries.get(&code) {
            Some(Some(link)) => self.by_link(*link),
            _ => &[]
        }
    }
}

