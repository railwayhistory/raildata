extern crate raildata;

pub use std::path::Path;
pub use raildata::load::load_tree;


fn main() {
    match load_tree(Path::new("../data").into()) {
        Ok(_) => println!("Ok."),
        Err(err) => {
            for line in err {
                println!("{}", line)
            }
        }
    }
}
