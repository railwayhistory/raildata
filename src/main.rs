extern crate raildata;

use std::{env, process};
use std::path::Path;
use std::time::Instant;
use raildata::load::load_tree;
use raildata::load::report::Stage;
use raildata::document::Data;

fn main() {
    let time = Instant::now();
    let path = env::args().nth(1).unwrap_or("../data".into());
    let store = match load_tree(Path::new(&path).into()) {
        Ok(store) => store,
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
                println!("{} errors.", err.len());
                for item in err.iter() {
                    println!("{}", item)
                }
            }
            process::exit(1);
        }
    };

    let store = match store.into_full_store() {
        Ok(store) => store,
        Err(mut err) => {
            err.sort();
            println!("{} errors.", err.len());
            for item in err.iter() {
                println!("{}", item)
            }
            process::exit(1);
        }
    };

    let mut lines = 0;
    let mut entities = 0;
    let mut paths = 0;
    let mut points = 0;
    let mut sources = 0;
    let mut structures = 0;

    for key in store.links() {
        match *key.data(&store) {
            Data::Line(_) => lines += 1,
            Data::Entity (_) => entities += 1,
            Data::Path(_) => paths += 1,
            Data::Point(_) => points += 1,
            Data::Source(_) => sources += 1,
            Data::Structure(_) => structures += 1,
        }
    }

    let time = Instant::now().duration_since(time);
    println!("Ok ({}.{:03} seconds)", time.as_secs(), time.as_millis());
    println!(
        "{} documents:",
        lines + entities + paths + points + sources + structures
    );
    println!("   {} lines", lines);
    println!("   {} entities", entities);
    println!("   {} paths", paths);
    println!("   {} points", points);
    println!("   {} sources", sources);
    println!("   {} structures", structures);
}
