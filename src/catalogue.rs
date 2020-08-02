use radix_trie::{Trie, TrieCommon};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use crate::document::DocumentLink;
use crate::types::List;


//------------ Catalogue -----------------------------------------------------

/// The meta-information we keep for finding documents.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Catalogue {
    /// Index of document names.
    ///
    /// This is the primary search index.
    names: Trie<String, List<(String, DocumentLink)>>,
}

impl Catalogue {
    pub fn new() -> Self {
        Self::default()
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
}

impl Catalogue {
    pub fn search_names(
        &self, prefix: &str, count: usize
    ) -> Vec<(&str, DocumentLink)> {
        let prefix = Self::normalize_name(prefix);
        self.names.get_raw_ancestor(&prefix).iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .flat_map(|(_, value)| value)
            .map(|(name, link)| (name.as_str(), *link))
            .take(count)
            .collect()
    }
}
