use std::sync::Arc;
use crate::catalogue::Catalogue;
use crate::store::FullStore;

pub struct State {
    store: FullStore,
    catalogue: Catalogue,
}

impl State {
    pub fn new(store: FullStore, catalogue: Catalogue) -> Self {
        State { store, catalogue }
    }

    pub fn new_arc(store: FullStore, catalogue: Catalogue) -> Arc<Self> {
        Arc::new(Self::new(store, catalogue))
    }

    pub fn store(&self) -> &FullStore {
        &self.store
    }

    pub fn catalogue(&self) -> &Catalogue {
        &self.catalogue
    }
}

