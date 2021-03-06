//! x86_64 specific structures
use super::*;

mod page_table;
pub use page_table::*;

mod recursive_mapper;
pub use recursive_mapper::*;
