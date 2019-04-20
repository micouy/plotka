//! JSON parsing.

use ::serde::Deserialize;
use serde_json as json;

use super::{Parser, ParseError, Record};

/// JSON parser.
pub struct JsonParser;

impl Parser for JsonParser {
    type Input = String;

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
