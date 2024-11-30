# raildata

A parser for the database of the Railway History Project. It provides both
a library to build upon and a binary that runs a check on the data set.

## Running the _raildata_ binary

Currently, you have to build the binary locally which requires a Rust
installation. See the [Install Rust](https://www.rust-lang.org/tools/install)
pages for more information.

Once you have Rust and Cargo, the easiest way to install the binary is
by simply running:

```
cargo install -f --git https://github.com/railwayhistory/raildata.git
```

After installing, you should be able to just run `raildata` in the
directory with your local copy of the database.

