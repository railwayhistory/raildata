extern crate raildata;

use std::path::Path;
use raildata::load::load_tree;


fn main() {
    match load_tree(Path::new("../data").into()) {
        Ok(_) => println!("Ok."),
        Err(mut err) => {
            println!("{} errors.", err.len());
            err.sort_by(|a, b| a.source().cmp(b.source()));
            for line in err {
                println!("{}", line)
            }
        }
    }
}
