#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

/// Filters for use in search queries
pub mod filter;
mod node;
/// Parser traits allow you to search different formats.
pub mod parser;
mod pattern;
/// Core functionality. Builds queries for searching
pub mod query;
mod soup;

pub use crate::{
    node::Node,
    pattern::Pattern,
    query::Queryable,
    soup::Soup,
};

/// Prelude: convenient import for all the user-facing APIs provided by the crate
pub mod prelude {
    pub use crate::*;
}
