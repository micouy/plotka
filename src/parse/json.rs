//! JSON parsing.
use ::serde_json as json;
use ::serde::{Deserialize, Deserializer};

use super::*;

struct JsonParser;

impl<'a> Parser<'a> for JsonParser {
    type Input = &'a str;
    type Error = json::Error;

    fn parse(&'a self, input: Self::Input) -> Result<Record<'a>, Self::Error> {
        let mut deserializer = json::Deserializer::from_str(input);

        Record::deserialize(&mut deserializer)
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
