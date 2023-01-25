
pub struct Document<Data, Meta> {
    data: Data,
    meta: Meta,
}

impl<Data, Meta> Document<Data, Meta> {
    pub fn new(data: Data, meta: Meta) -> Self {
        Document { data, meta }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}

