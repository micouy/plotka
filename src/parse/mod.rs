//! Input parsing.

use std::{collections::HashMap, fmt};

pub mod csv;
pub mod json;
pub mod record;

use self::record::Record;
use crate::storage::Number;

/// Parser options.
pub enum Opts {
    /// Create a CSV parser from headers.
    Csv(Vec<String>),
    /// Create a JSON parser.
    Json,
}

/// Parse error.
///
/// This type provides no information about the cause of the error because
/// it is irrelevant to the user. The file formats supported by Plotka are popular,
/// simple and well documented.
#[derive(Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}

/// A record parser.
///
/// The `'static` lifetime is required in order to implement
/// the [`Handler`][actix::Handler] trait on the
/// [`Server`][crate::server::Server].
pub trait Parser: 'static {
    /// Expected input.
    type Input;

    /// Parse the input.
    fn parse<'a>(
        &'a self,
        input: &'a Self::Input,
    ) -> Result<Record<'a>, ParseError>;
}
