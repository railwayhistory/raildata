
use radix_trie::{Trie, TrieCommon};
use unicode_normalization::UnicodeNormalization;
use crate::load::report::{Report, Reporter, Stage};
use crate::store::{DocumentLink, FullStore};
use crate::types::List;


//------------ CatalogueBuilder ----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct CatalogueBuilder(Catalogue);

impl CatalogueBuilder {
    pub fn insert_name(&mut self, name: String, link: DocumentLink) {
        let term = Catalogue::normalize_name(&name);
        if let Some(value) = self.0.names.get_mut(&term) {
            value.push((name, link))
        }
        else {
            self.0.names.insert(term, List::with_value((name, link)));
        }
    }
}


//------------ Catalogue -----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Catalogue {
    names: Trie<String, List<(String, DocumentLink)>>,
}

impl Catalogue {
    pub fn generate(store: &FullStore) -> Result<Self, Report> {
        let report = Reporter::new();
        let mut ok = true;
        let builder = {
            let mut stage_report = report.clone().stage(Stage::Catalogue);
            let mut builder = CatalogueBuilder::default();
            for link in store.links() {
                if link.data(store).catalogue(
                    &mut builder, store, &mut stage_report
                ).is_err() {
                    ok = false;
                }
            }
            builder
        };
        if ok {
            Ok(builder.0)
        }
        else {
            Err(report.unwrap())
        }
    }

    pub fn search_name(
        &self, prefix: &str
    ) -> impl Iterator<Item = (&str, DocumentLink)> {
        let prefix = Self::normalize_name(prefix);
        self.names.get_raw_ancestor(&prefix).iter()
            .filter(move |(key, _)| key.starts_with(&prefix))
            .flat_map(|(_, value)| value)
            .map(|(name, link)| (name.as_str(), *link))
    }

    fn normalize_name(name: &str) -> String {
        name.nfd()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect()
    }
}

