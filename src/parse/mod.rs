//! Input parsing.

use std::{collections::HashMap, fmt, io};

pub mod csv;
pub mod json;
pub mod record;

use crate::storage::Number;
use crate::parse::record::Record;

/// Parser settings.
pub enum ParserSettings {
    /// Create a CSV parser from headers and delimiter.
    Csv {
        headers: Vec<String>,
        delimiter: Option<u8>,
    },
    /// Create a JSON parser.
    Json,
}

/// Parse error.
///
/// This type provides no information about the cause of the error because
/// it is irrelevant to the user. The file formats supported by Plotka are
/// popular, simple and well documented.
#[derive(Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}

#[derive(Debug)]
pub struct ReadError;

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "read error")
    }
}

/// Record parser.
///
/// The `'static` lifetime is required in order to implement
/// the [`Handler`][actix::Handler] trait on the
/// [`Server`][crate::server::Server].
pub trait Parser<R>: 'static where R: io::Read {
    /// Expected input.
    type Input: Send;

    /// Reader settings.
    type Settings: Send;

    /// Iterator yielding [`Self::Input`][Parser::Input].
    type Reader: Iterator<Item = Result<Self::Input, ReadError>>;

    /// Wrap provided reader into [`Self::Reader`][Parser::Reader].
    fn wrap_reader(reader: R, settings: Self::Settings) -> Self::Reader;

    /// Parse the input.
    fn parse<'a>(
        &'a self,
        input: &'a Self::Input,
    ) -> Result<Record<'a>, ParseError>;
}
