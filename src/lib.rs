#![warn(missing_docs)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::new_without_default)]

//! Plotka's core functionality.

mod error;

pub mod parse;
pub mod server;
pub mod storage;

use self::parse::record::Record;

pub use self::error::Error;
