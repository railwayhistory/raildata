extern crate raildata;

use std::path::Path;
use raildata::load::load_tree;


fn main() {
    match load_tree(Path::new("../data").into()) {
        Ok(_) => println!("Ok."),
        Err(mut err) => {
            err.sort();
            println!("{} errors.", err.len());
            println!("{}", err)
        }
    }
}
