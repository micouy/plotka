//! CSV parsing.

//!
use csv::{Reader as CsvReader, ReaderBuilder as CsvReaderBuilder};

use std::io::prelude::*;

use super::{Parser, ParserCreationError, Record};

fn create_csv_reader<R>(
    reader: R,
    has_headers: bool,
    mb_delimiter: Option<u8>,
) -> CsvReader<R>
where
    R: Read,
{
    CsvReaderBuilder::new()
        .delimiter(mb_delimiter.unwrap_or(b','))
        .has_headers(has_headers)
        .from_reader(reader)
}

fn extract_headers<R>(
    mb_headers: Option<Vec<String>>,
    csv_reader: &mut CsvReader<R>,
) -> Result<Vec<String>, ParserCreationError>
where
    R: Read,
{
    // Try to get headers from the `mb_headers` arg, then from the reader.
    // If there are none specified, return an error.
    mb_headers
        .or_else(|| {
            csv_reader
                .headers()
                .map(|headers| {
                    headers.iter().map(|header| header.to_string()).collect()
                })
                .ok()
        })
        .ok_or_else(|| ParserCreationError::NoCsvHeaders)
}

/// Create a CSV parser from a reader, headers and a delimiter.
pub fn create_csv_parser<R>(
    reader: R,
    mb_headers: Option<Vec<String>>,
    mb_delimiter: Option<u8>,
) -> Result<(impl Parser, Vec<String>), ParserCreationError>
where
    R: Read,
{
    let mut csv_reader =
        create_csv_reader(reader, mb_headers.is_none(), mb_delimiter);
    let headers = extract_headers(mb_headers, &mut csv_reader)?;

    // Create the actual parser - an iterator yielding `Record`s from each line
    // of input.
    let parser = {
        let headers: csv::StringRecord = headers.clone().into();

        csv_reader.into_records().map(|record| record.unwrap()).map(
            move |record| {
                record.deserialize::<Record>(Some(&headers.clone())).unwrap()
            },
        )
    };

    Ok((parser, headers))
}

#[cfg(test)]
mod test {
    use ::pretty_assertions::assert_eq;

    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_parse_csv() {
        let headers = None;
        let delimiter = None;

        let contents = "a,b\n1,2\n11,12\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, headers) =
            create_csv_parser(reader, headers, delimiter).unwrap();

        assert_eq!(headers, vec!["a", "b"]);

        let record_1 = parser.next().unwrap();

        assert_eq!(record_1["a"], 1.0);
        assert_eq!(record_1["b"], 2.0);

        let record_2 = parser.next().unwrap();

        assert_eq!(record_2["a"], 11.0);
        assert_eq!(record_2["b"], 12.0);

        assert!(parser.next().is_none());
    }

    #[test]
    fn test_parse_csv_with_headers() {
        let headers = Some(vec!["a".to_string(), "b".to_string()]);
        let delimiter = None;

        let contents = "1,2\n11,12\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, headers) =
            create_csv_parser(reader, headers, delimiter).unwrap();

        assert_eq!(headers, vec!["a", "b"]);

        let record_1 = parser.next().unwrap();

        assert_eq!(record_1["a"], 1.0);
        assert_eq!(record_1["b"], 2.0);

        let record_2 = parser.next().unwrap();

        assert_eq!(record_2["a"], 11.0);
        assert_eq!(record_2["b"], 12.0);

        assert!(parser.next().is_none());
    }

    #[test]
    fn test_parse_csv_with_delimiter() {
        let headers = None;
        let delimiter = Some(b';');

        let contents = "a;b\n1;2\n11;12\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, headers) =
            create_csv_parser(reader, headers, delimiter).unwrap();

        assert_eq!(headers, vec!["a", "b"]);

        let record_1 = parser.next().unwrap();

        assert_eq!(record_1["a"], 1.0);
        assert_eq!(record_1["b"], 2.0);
    }

    #[test]
    #[should_panic]
    fn test_parse_csv_panic_on_non_number_data() {
        let headers = Some(vec!["a".to_string(), "b".to_string()]);
        let delimiter = None;

        let contents = "h,i\nj,k\nl,m\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, _headers) =
            create_csv_parser(reader, headers, delimiter).unwrap();

        let _record = parser.next();
    }
}
