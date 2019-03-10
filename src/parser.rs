//! Input parsing.

use std::{collections::HashMap, io::prelude::*};

use csv::ReaderBuilder;

type Record = HashMap<String, f64>;

#[derive(Debug)]
enum ParserError {
    NoCsvHeaders,
}

enum Opts {
    Csv(Option<Vec<String>>, Option<u8>),
}

fn generate_parser<R>(
    reader: R,
    opts: Opts,
) -> Result<(impl Iterator<Item = Record>, Vec<String>), ParserError>
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

fn generate_csv_parser<R>(
    reader: R,
    mb_headers: Option<Vec<String>>,
    mb_delimiter: Option<u8>,
) -> Result<(impl Iterator<Item = Record>, Vec<String>), ParserError>
where
    R: Read,
{
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(mb_delimiter.unwrap_or(b','))
        .has_headers(match mb_headers {
            Some(_) => false,
            None => true,
        })
        .from_reader(reader);

    let headers = mb_headers
        .or_else(|| {
            csv_reader
                .headers()
                .map(|headers| {
                    headers.iter().map(|header| header.to_string()).collect()
                })
                .ok()
        })
        .ok_or_else(|| ParserError::NoCsvHeaders)?;

    let parser = {
        let headers = headers.clone();

        csv_reader.into_records().map(|record| record.unwrap()).map(
            move |record| {
                record
                    .deserialize::<HashMap<String, f64, _>>(Some(
                        &headers.clone().into(),
                    ))
                    .unwrap()
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
        let opts = Opts::Csv(headers, delimiter);

        let contents = "a,b\n1,2\n11,12\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, headers) = generate_parser(reader, opts).unwrap();

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
        let opts = Opts::Csv(headers, delimiter);

        let contents = "1,2\n11,12\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, headers) = generate_parser(reader, opts).unwrap();

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
    #[should_panic]
    fn test_parse_csv_panic_on_non_number_data() {
        let headers = Some(vec!["a".to_string(), "b".to_string()]);
        let delimiter = None;
        let opts = Opts::Csv(headers, delimiter);

        let contents = "h,i\nj,k\nl,m\n";
        let reader = BufReader::new(contents.as_bytes());

        let (mut parser, _headers) = generate_parser(reader, opts).unwrap();

        let _record = parser.next();
    }
}
