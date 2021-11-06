mod binary;
pub use binary::*;

mod boolean;
pub use boolean::*;

mod date_time;
pub use date_time::*;

mod date_time_utils;

mod numbers;
pub use numbers::*;

mod qname;
pub use qname::*;

mod strings;
pub use strings::*;

use std::fmt::Debug;

pub(crate) trait Value: Debug {

}