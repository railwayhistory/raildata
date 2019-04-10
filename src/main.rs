extern crate raildata;

use std::env;
use std::path::Path;
use raildata::load::load_tree;
use raildata::store::StoredDocument;


fn main() {
    let path = env::args().nth(1).unwrap_or("../data".into());
    let store = match load_tree(Path::new(&path).into()) {
        Ok(store) => store,
        Err(mut err) => {
            err.sort();
            println!("{} errors.", err.len());
            for item in err.iter() {
                println!("{}", item)
            }
            ::std::process::exit(1);
        }
    };

    for item in store.iter_from("line.de") {
        if !item.key().starts_with("line.de") {
            break;
        }
        let item = match item {
            StoredDocument::Line(item) => item,
            _ => continue
        };
        println!("{}", item.key());
        let points = item.points();
        let mut points = points.iter();
        if let Some(point) = points.next() {
            let point = point.resolve(&store);
            println!("    {}", point.key())
        }
        else {
            continue
        }
        let mut last = None;
        for point in points {
            let point = point.resolve(&store);
            if point.junction() {
                last = None;
                println!("    {}", point.key())
            }
            else {
                last = Some(point)
            }
        }
        if let Some(point) = last {
            println!("    {}", point.key())
        }
    }
}
