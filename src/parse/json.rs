//! JSON parsing.

use ::serde::Deserialize;
use serde_json as json;

use std::io::{self, BufRead, BufReader, Lines};

use super::{record::Record, ParseError, Parser, ReadError};

/// JSON parser.
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct JsonReader<R>(Lines<BufReader<R>>)
where
    R: io::Read;

impl<R> JsonReader<R>
where
    R: io::Read,
{
    pub fn new(reader: R) -> Self {
        Self(BufReader::new(reader).lines())
    }
}

impl<R> Iterator for JsonReader<R>
where
    R: io::Read,
{
    type Item = Result<String, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|line| line.map_err(|_| ReadError {}))
    }
}

impl<R> Parser<R> for JsonParser
where
    R: io::Read,
{
    type Input = String;

    type Settings = ();

    type Reader = JsonReader<R>;

    fn wrap_reader(reader: R, settings: Self::Settings) -> Self::Reader {
        JsonReader::new(reader)
    }

    fn parse<'a>(
        &'a self,
        input: &'a Self::Input,
    ) -> Result<Record, ParseError> {
        let mut deserializer = json::Deserializer::from_str(&input);

        Record::deserialize(&mut deserializer).map_err(|_| ParseError {})
    }
}

#[cfg(test)]
mod test {
    use ::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_json() {
        let contents = r#"{ "a": 1, "b": 2 }"#;
        let parser = JsonParser {};

        let record = parser.parse(contents).unwrap();

        assert_eq!(record["a"], Number::Int(1));
        assert_eq!(record["b"], Number::Int(2));
    }

    #[test]
    fn test_parse_json_from_floats() {
        let contents = r#"{ "a": 1.0, "b": 2.0 }"#;
        let parser = JsonParser {};

        let record = parser.parse(contents).unwrap();

        assert_eq!(record["a"], Number::Float(1.0));
        assert_eq!(record["b"], Number::Float(2.0));
    }
}
