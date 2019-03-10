//! Input parsing.

use std::{collections::HashMap, io::prelude::*};

mod csv;

use self::csv::*;

pub type Record = HashMap<String, f64>;

#[derive(Debug)]
pub enum ParserCreationError {
    NoCsvHeaders,
}

pub enum Opts {
    Csv(Option<Vec<String>>, Option<u8>),
}

pub fn generate_parser<R>(
    reader: R,
    opts: Opts,
) -> Result<(impl Iterator<Item = Record>, Vec<String>), ParserCreationError>
where
    R: Read,
{
    #[allow(unreachable_patterns)]
    match opts {
        Opts::Csv(headers, delimiter) =>
            generate_csv_parser(reader, headers, delimiter),
        _ => unimplemented!(),
    }
}
