#![feature(const_fn_trait_bound)]
#![feature(const_precise_live_drops)]
#![feature(const_panic)]
#![feature(map_first_last)]
#![feature(destructuring_assignment)]

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