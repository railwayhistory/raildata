extern crate raildata;

use std::path::Path;
use raildata::load::load_tree;


fn main() {
    match load_tree(Path::new("../data").into()) {
        Ok(doc) => println!("Ok. {} documents.", doc.len()),
        Err(mut err) => {
            err.sort();
            println!("{} errors.", err.len());
            for item in err.iter() {
                println!("{}", item)
            }
        }
    }
}
