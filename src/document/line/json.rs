use crate::store::FullStore;
use super::Document;

impl<'a> Document<'a> {
    pub fn json(self, _store: &FullStore) -> String {
        self.data().common.json(|json| {
            json.member_str("type", "line");
        })
    }
}

