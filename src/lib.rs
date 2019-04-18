extern crate crossbeam;
#[macro_use] extern crate derive_more;
extern crate ignore;
extern crate osmxml;
extern crate rayon;
extern crate yaml_rust;
extern crate url;

#[macro_use] pub mod types;
pub mod document;
pub mod library;
pub mod load;
pub mod store;
