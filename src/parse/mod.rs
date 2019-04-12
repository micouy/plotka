//! Input parsing.

use std::{collections::HashMap, fmt};

pub mod csv;
pub mod record;
pub mod json;

use self::record::Record;
use crate::storage::Number;

/// Parser options.
pub enum Opts {
    /// Create a CSV parser from headers.
    Csv(Vec<String>),
    /// Create a JSON parser.
    Json,
}

/// A record parser.
pub trait Parser<'a> {
    /// Expected input.
    type Input;

    /// Parse error.
    type Error: fmt::Display;

    /// Parse the input.
    fn parse(
        &'a self,
        input: Self::Input,
    ) -> Result<Record<'a>, Self::Error>;
}
