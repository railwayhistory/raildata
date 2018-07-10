use super::document::Document;
use super::store::Store;


//------------ UpdateStore ---------------------------------------------------

#[derive(Debug)]
pub struct UpdateStore {
    documents: Vec<Option<Document>>,
}

impl UpdateStore {
    pub fn from_documents<I: Iterator<Item=Document>>(iter: I) -> Self {
        UpdateStore {
            documents: iter.map(Some).collect()
        }
    }

    pub fn into_store(self) -> Store {
        Store::from_documents(
            self.documents.into_iter().map(Option::unwrap)
        )
    }
}

impl UpdateStore {
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn take_document(&mut self, pos: usize) -> Document {
        self.documents.get_mut(pos).unwrap().take().unwrap()
    }

    pub fn return_document(&mut self, pos: usize, document: Document) {
        let place = self.documents.get_mut(pos).unwrap();
        if place.is_some() {
            panic!("trying to return exisiting document");
        }
        *place = Some(document);
    }

    pub fn update<F>(&mut self, pos: usize, op: F)
    where F: FnOnce(&mut Document) {
        op(self.documents.get_mut(pos).unwrap().as_mut().unwrap())
    }
}
