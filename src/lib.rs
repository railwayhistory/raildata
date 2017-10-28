#![recursion_limit="128"]

extern crate osmxml;
extern crate ignore;
extern crate url;
extern crate yaml_rust;

#[macro_use] mod macros;
pub mod document;
pub mod index;
pub mod links;
pub mod load;
pub mod types;
