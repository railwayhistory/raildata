use std::sync::RwLock;
use super::document::Document;
use super::store::Store;


//------------ UpdateStore ---------------------------------------------------

#[derive(Debug)]
pub struct UpdateStore {
    documents: Vec<RwLock<Document>>,
}

impl UpdateStore {
    pub fn from_documents<I: Iterator<Item=Document>>(iter: I) -> Self {
        UpdateStore {
            documents: iter.map(RwLock::new).collect()
        }
    }

    pub fn into_store(self) -> Store {
        Store::from_documents(
            self.documents.into_iter().map(|item| item.into_inner().unwrap())
        )
    }
}
