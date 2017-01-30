use ::collection::{CollectionBuilder, DocumentRef, DocumentGuard};
use ::load::error::{ErrorGatherer};
use ::load::yaml::{FromYaml, Item, Mapping, Sequence, ValueItem};
use super::document::DocumentType;


//------------ Structure -----------------------------------------------------

pub struct Structure {
    key: String
}

impl Structure {
    pub fn from_yaml(key: String, mut item: Item<Mapping>,
                     collection: &mut CollectionBuilder,
                     errors: &ErrorGatherer) -> Result<Self, ()> {
        Ok(Structure {
            key: key
        })
    }
}

impl Structure {
    pub fn key(&self) -> &str {
        &self.key
    }
}


//------------ StructureRef --------------------------------------------------

pub struct StructureRef(DocumentRef);

impl StructureRef {
    pub fn get(&self) -> DocumentGuard<Structure> {
        self.0.get()
    }
}

impl FromYaml for StructureRef {
    fn from_yaml(item: ValueItem, collection: &mut CollectionBuilder,
                 errs: &ErrorGatherer) -> Result<Self, ()> {
        let item = item.into_string_item(errs)?;
        Ok(StructureRef(collection.ref_doc(item.value(), item.source(),
                                      DocumentType::Structure)))
    }
}

