#![feature(map_first_last)]

mod namespaces;
mod parser;
mod fns;
mod eval;
mod values;
mod tree;
mod serialization;

mod tests;
mod xqts;

#[macro_use]
extern crate lazy_static;