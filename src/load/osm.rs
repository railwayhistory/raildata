use osmxml::read::read_xml;
use ::document::path::Path;
use ::document::store::DocumentStoreBuilder;
use ::types::{IntoMarked, Location};


//------------ load_osm_file -------------------------------------------------

pub fn load_osm_file<R: io::Read>(
    read: &mut R,
    docs: &mut DocumentStoreBuilder,
    report: &mut PathReporter
) {
    let mut osm = match read_xml(read) {
        Ok(osm) => osm,
        Err(err) => {
            report.error(err.unmarked());
            return;
        }
    };
    
    // Swap out the relations so we don’t hold a mutable reference to
    // `osm` while draining the relations.
    let mut relations = HashSet::new();
    mem::swap(osm.relations_mut(), &mut relations);
    for relation in relations.drain() {
        match Path::from_osm(relation, &osm, docs, report) {
            Ok(path) => {
                docs.insert(path.key().clone(), path, Location::None, report)
            }
            Err(Some(key)) => {
                docs.insert_broken::<Path>(key, Location::None, report)
            }
            Err(None) => { }
        }
    }
}

