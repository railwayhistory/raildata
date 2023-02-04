use std::process;
#[cfg(feature = "http")]
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use raildata::catalogue::Catalogue;
use raildata::document::Data;
use raildata::load::load_tree;
use raildata::load::report::Stage;
use raildata::store::DataStore;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the data directory.
    path: PathBuf,

    /// Start the HTTP server listening on given addr.
    #[cfg(feature = "http")]
    #[arg(long, value_name = "ADDR")]
    http: Option<SocketAddr>,

    /// Do a quick parse and exit.
    #[arg(long, short, conflicts_with = "http")]
    quick: bool,

    /// Verbose output.
    #[arg(long, short)]
    verbose: bool,
}

fn print_stats(store: &DataStore) {
    let mut lines = 0;
    let mut entities = 0;
    let mut paths = 0;
    let mut points = 0;
    let mut sources = 0;
    let mut structures = 0;

    for key in store.links() {
        match *key.data(store) {
            Data::Line(_) => lines += 1,
            Data::Entity (_) => entities += 1,
            Data::Path(_) => paths += 1,
            Data::Point(_) => points += 1,
            Data::Source(_) => sources += 1,
            Data::Structure(_) => structures += 1,
        }
    }
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

fn main() {
    let args = Args::parse();

    let time = Instant::now();
    let store = match load_tree(&args.path) {
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
    if args.verbose {
        println!(
            "Parsing: {:.3} s",
            Instant::now().duration_since(time).as_secs_f32()
        );
    }
    if args.quick {
        if args.verbose {
            print_stats(&store);
        }
        else {
            println!("Ok.");
        }
        process::exit(1);
    }

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

    #[allow(unused_variables)]
    let catalogue = match Catalogue::generate(&store) {
        Ok(catalogue) => catalogue,
        Err(mut err) => {
            err.sort();
            println!("{} errors.", err.len());
            for item in err.iter() {
                println!("{}", item)
            }
            process::exit(1);
        }
    };

    println!("Ok.");
    if args.verbose {
        let time = Instant::now().duration_since(time);
        println!("Total: {:.3} s.", time.as_secs_f32());
        print_stats(store.as_ref());
    }

    #[cfg(feature = "http")]
    {
        use tokio::runtime::Runtime;
        use raildata::http::state::State;
        use raildata::http::api;

        if let Some(addr) = args.http {
            let rt = Runtime::new().unwrap();

            rt.block_on(
                api::serve(
                    addr,
                    State::new_arc(store, catalogue),
                )
            );
        }
    }
}
