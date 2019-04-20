//! CSV parsing.

use std::fmt;

use super::{Parser, ParseError, Record};

/// CSV parser.
#[derive(Debug)]
pub struct CsvParser {
    headers: csv::StringRecord,
}

impl CsvParser {
    /// Create a CSV parser from headers.
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers: headers.into(),
        }
    }
}

impl Parser for CsvParser {
    type Input = csv::StringRecord;

    fn parse<'a>(
        &'a self,
        input: &'a Self::Input,
    ) -> Result<Record<'a>, ParseError> {
        input
            .deserialize(Some(&self.headers))
            .map_err(|_| ParseError {})
    }
}

#[cfg(test)]
mod test {
    use ::pretty_assertions::assert_eq;

    use std::io::BufReader;

    use super::*;

    use crate::storage::Number;

    fn create_csv_reader<R>(
        reader: R,
        mb_delimiter: Option<u8>,
        ignore_first_line: bool,
    ) -> CsvReader<R>
    where
        R: Read,
    {
        let reader = CsvReaderBuilder::new()
            .delimiter(mb_delimiter.unwrap_or(b','))
            .has_headers(ignore_first_line)
            .from_reader(reader);

        reader
    }

    #[test]
    fn test_parse_csv() {
        let headers = vec!["a".to_string(), "b".to_string()];
        let delimiter = None;

        let contents = "1,2\n11,12\n";
        let reader = BufReader::new(contents.as_bytes());
        let mut reader = create_csv_reader(reader, delimiter, false);
        let mut records = reader.records();

        let parser = CsvParser::new(headers);

        let input = records.next().unwrap().unwrap();
        let record_1 = parser.parse(&input).unwrap();

        assert_eq!(record_1["a"], Number::Int(1));
        assert_eq!(record_1["b"], Number::Int(2));

        let input = records.next().unwrap().unwrap();
        let record_2 = parser.parse(&input).unwrap();

        assert_eq!(record_2["a"], Number::Int(11));
        assert_eq!(record_2["b"], Number::Int(12));
    }

    #[test]
    fn test_parse_csv_from_floats() {
        let headers = vec!["a".to_string(), "b".to_string()];
        let delimiter = None;

        let contents = "1.0,2.0\n11.0,12.0\n";
        let reader = BufReader::new(contents.as_bytes());
        let mut reader = create_csv_reader(reader, delimiter, false);
        let mut records = reader.records();

        let parser = CsvParser::new(headers);

        let input = records.next().unwrap().unwrap();
        let record_1 = parser.parse(&input).unwrap();

        assert_eq!(record_1["a"], Number::Float(1.0));
        assert_eq!(record_1["b"], Number::Float(2.0));

        let input = records.next().unwrap().unwrap();
        let record_2 = parser.parse(&input).unwrap();

        assert_eq!(record_2["a"], Number::Float(11.0));
        assert_eq!(record_2["b"], Number::Float(12.0));
    }

    #[test]
    fn test_parse_csv_with_delimiter() {
        let headers = vec!["a".to_string(), "b".to_string()];
        let delimiter = Some(b';');

        let contents = "a;b\n1;2";
        let reader = BufReader::new(contents.as_bytes());
        let mut reader = create_csv_reader(reader, delimiter, true);
        let mut records = reader.records();

        let parser = CsvParser::new(headers);

        let input = records.next().unwrap().unwrap();
        let record_1 = parser.parse(&input).unwrap();

        assert_eq!(record_1["a"], Number::Int(1));
        assert_eq!(record_1["b"], Number::Int(2));
    }

    #[test]
    #[should_panic]
    fn test_parse_csv_panic_on_non_number_data() {
        let headers = vec!["a".to_string(), "b".to_string()];
        let delimiter = None;

        let contents = "h,i\nj,k\nl,m\n";
        let reader = BufReader::new(contents.as_bytes());
        let mut reader = create_csv_reader(reader, delimiter, true);
        let mut records = reader.records();

        let parser = CsvParser::new(headers);

        let input = records.next().unwrap().unwrap();
        let _record = parser.parse(&input).unwrap();
    }
}
