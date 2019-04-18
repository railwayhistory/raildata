extern crate raildata;

use std::env;
use std::path::Path;
use std::time::Instant;
use raildata::load::load_tree;
use raildata::load::report::Stage;
use raildata::document::Document;

fn main() {
    let time = Instant::now();
    let path = env::args().nth(1).unwrap_or("../data".into());
    let library = match load_tree(Path::new(&path).into()) {
        Ok(library) => library,
        Err(mut err) => {
            err.sort();

            if err.has_stage(Stage::Parse) {
                println!("{} errors.", err.stage_count(Stage::Parse));
                for item in err.iter() {
                    if item.stage() == Stage::Parse {
                        println!("{}", item)
                    }
                }
            }
            else {
                println!("{} errors.", err.stage_count(Stage::Translate));
                for item in err.iter() {
                    println!("{}", item)
                }
            }
            ::std::process::exit(1);
        }
    };

    let mut lines = 0;
    let mut organizations = 0;
    let mut paths = 0;
    let mut points = 0;
    let mut sources = 0;
    let mut structures = 0;

    for doc in library.iter() {
        match *doc {
            Document::Line(_) => lines += 1,
            Document::Organization(_) => organizations += 1,
            Document::Path(_) => paths += 1,
            Document::Point(_) => points += 1,
            Document::Source(_) => sources += 1,
            Document::Structure(_) => structures += 1,
        }
    }

    let time = Instant::now().duration_since(time);
    println!("Ok ({}.{:03} seconds)", time.as_secs(), time.as_millis());
    println!(
        "{} documents:",
        lines + organizations + paths + points + sources + structures
    );
    println!("   {} lines", lines);
    println!("   {} organizations", organizations);
    println!("   {} paths", paths);
    println!("   {} points", points);
    println!("   {} sources", sources);
    println!("   {} structures", structures);
}
