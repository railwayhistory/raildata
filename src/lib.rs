#![recursion_limit="128"]

#[macro_use] extern crate failure_derive;
extern crate failure;
extern crate osmxml;
extern crate ignore;
extern crate url;
extern crate yaml_rust;

//#[macro_use] mod macros;
#[macro_use] pub mod types;
pub mod document;
pub mod load;

