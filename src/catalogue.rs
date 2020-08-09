use std::collections::HashMap;
use radix_trie::{Trie, TrieCommon};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use crate::document::{Document, DocumentLink};
use crate::document::common::DocumentType;
use crate::library::LibraryMut;
use crate::types::List;


//------------ Catalogue -----------------------------------------------------

/// The meta-information we keep for finding documents.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Catalogue {
    /// Index of document names.
    ///
    /// This is the primary search index.
    names: Trie<String, List<(String, DocumentLink)>>,

    /// Index of countries in alphabetical order of country code.
    countries: Vec<DocumentLink>,

    /// The number of documents for the various types.
    doc_num: HashMap<DocumentType, usize>,
}

impl Catalogue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finalize(&mut self, library: &LibraryMut) {
        self.countries.sort_unstable_by(|left, right| {
            library.resolve_mut(*left).key().cmp(
                library.resolve_mut(*right).key()
            )
        })
    }

    pub fn insert_name(&mut self, name: String, link: DocumentLink) {
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

    pub fn push_country(&mut self, link: DocumentLink) {
        self.countries.push(link)
    }

    pub fn register(&mut self, doc: &Document) {
        *self.doc_num.entry(doc.doctype()).or_insert(0) += 1;
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
    ) -> impl Iterator<Item = DocumentLink> + 'a {
        self.countries.iter().cloned()
    }

    pub fn doc_nums<'a>(
        &'a self
    ) -> impl Iterator<Item = (DocumentType, usize)> + 'a {
        self.doc_num.iter().map(|(k, v)| (*k, *v))
    }
}

