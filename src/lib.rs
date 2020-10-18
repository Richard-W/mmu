#![cfg_attr(not(test), no_std)]

#[macro_use]
mod macros;

mod result;
pub use result::*;

mod types;
pub use types::*;

pub mod amd64;
