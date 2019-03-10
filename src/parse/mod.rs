//! Input parsing.

use std::{collections::HashMap, io::prelude::*};

pub mod csv;

use self::csv::*;

/// A record is a hashmap with [`String`] fields storing [`f64`] values.
pub type Record = HashMap<String, f64>;

/// Parser creation error.
#[derive(Debug)]
pub enum ParserCreationError {
    /// CSV headers have not been found in input or in args.
    NoCsvHeaders,
}

/// Parser options.
pub enum Opts {
    /// Create a CSV parser from headers and a delimiter.
    Csv(Option<Vec<String>>, Option<u8>),
}

/// Create a parser from reader and options.
///
/// To learn more, see [`create_csv_parser`].
pub fn create_parser<R>(
    reader: R,
    opts: Opts,
) -> Result<(impl Iterator<Item = Record>, Vec<String>), ParserCreationError>
where
    R: Read,
{
    #[allow(unreachable_patterns)]
    match opts {
        Opts::Csv(headers, delimiter) =>
            create_csv_parser(reader, headers, delimiter),
        _ => unimplemented!(),
    }
}
