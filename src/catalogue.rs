
use crate::load::report::{Report, Reporter, Stage};
use crate::store::FullStore;


//------------ CatalogueBuilder ----------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct CatalogueBuilder(Catalogue);


//------------ Catalogue -----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Catalogue;

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
}

