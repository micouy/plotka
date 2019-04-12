//! Record deserialization.

use ::serde::{
    de::{
        self,
        MapAccess,
        Visitor,
    },
    Deserialize,
    Deserializer,
};

use std::{borrow::Cow, fmt, marker::PhantomData};

use super::*;

/// Deserialization error.
#[derive(Debug)]
pub enum DeserError {
    /// Cannot parse the input.
    Parse,
    /// The `next` method called on an already empty itearator.
    NextOnEmptyIter,
}

impl fmt::Display for DeserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "deser error")
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor {}

        impl NumberVisitor {
            fn parse_str(v: &str) -> Result<Number, serde::de::value::Error> {
                if let Ok(int) = v.parse::<i64>() {
                    Ok(Number::Int(int))
                } else if let Ok(float) = v.parse::<f64>() {
                    Ok(Number::Float(float))
                } else {
                    Err(de::Error::custom(DeserError::Parse))
                }
            }
        }

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number, string-like type or bytes a number can be parsed from")
            }

            fn visit_i8<E>(self, int: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(i64::from(int)))
            }

            fn visit_i32<E>(self, int: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(i64::from(int)))
            }

            fn visit_i64<E>(self, int: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(int))
            }

            fn visit_u8<E>(self, uint: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(i64::from(uint)))
            }

            fn visit_u32<E>(self, uint: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(i64::from(uint)))
            }

            fn visit_u64<E>(self, uint: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Int(uint as i64))
            }

            fn visit_f32<E>(self, float: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Float(f64::from(float)))
            }

            fn visit_f64<E>(self, float: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Number::Float(float))
            }

            fn visit_borrowed_str<E>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::parse_str(v)
                    .map_err(|_| de::Error::custom(DeserError::Parse))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::parse_str(v.as_str())
                    .map_err(|_| de::Error::custom(DeserError::Parse))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::parse_str(v)
                    .map_err(|_| de::Error::custom(DeserError::Parse))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let d = v
                    .to_digit(10)
                    .ok_or_else(|| de::Error::custom(DeserError::Parse))?;

                Ok(Number::Int(i64::from(d)))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = std::str::from_utf8(v).map_err(de::Error::custom)?;

                Self::parse_str(s)
                    .map_err(|_| de::Error::custom(DeserError::Parse))
            }

            fn visit_borrowed_bytes<E>(
                self,
                v: &'de [u8],
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = std::str::from_utf8(v)
                    .map_err(|_| de::Error::custom(DeserError::Parse))?;

                Self::parse_str(s).map_err(de::Error::custom)
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = std::str::from_utf8(&v)
                    .map_err(|_| de::Error::custom(DeserError::Parse))?;

                Self::parse_str(s).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(NumberVisitor {})
    }
}

struct CowWrapper<'a>(pub Cow<'a, str>);

impl<'a, 'de: 'a> Deserialize<'de> for CowWrapper<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CowStrVisitor<'a>(PhantomData<&'a str>);

        impl<'a, 'de: 'a> Visitor<'de> for CowStrVisitor<'a> {
            type Value = Cow<'a, str>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number, string-like type or bytes a number can be parsed from")
            }

            fn visit_borrowed_str<E>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Borrowed(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Owned(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Owned(v.to_owned()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Owned(
                    String::from_utf8(v.to_vec()).map_err(de::Error::custom)?,
                ))
            }

            fn visit_borrowed_bytes<E>(
                self,
                v: &'de [u8],
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Borrowed(
                    std::str::from_utf8(v).map_err(de::Error::custom)?,
                ))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Cow::Owned(String::from_utf8(v).map_err(de::Error::custom)?))
            }
        }

        deserializer
            .deserialize_str(CowStrVisitor(PhantomData))
            .map(|cow| CowWrapper(cow))
    }
}

/// A wrapper around a hashmap.
pub struct Record<'a>(pub HashMap<Cow<'a, str>, Number>);

impl<'a, 'de: 'a> Deserialize<'de> for Record<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RecordVisitor<'a>(PhantomData<Cow<'a, str>>);

        impl<'a, 'de: 'a> Visitor<'de> for RecordVisitor<'a> {
            type Value = Record<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map of numbers with strings as keys")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                (0..)
                    .map(|_| map.next_entry::<CowWrapper, Number>())
                    .take_while(|entry| match entry {
                        Ok(None) => false,
                        _ => true,
                    })
                    .map(|result| {
                        result.map(|entry| {
                            let (key, number) = entry.unwrap();

                            (key.0, number)
                        })
                    }) // already checked for Ok(None)
                    .collect::<Result<HashMap<_, _, _>, _>>()
                    .map(|hashmap| Record(hashmap))
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_map(RecordVisitor(PhantomData))
    }
}

impl<'a, S> std::ops::Index<S> for Record<'a> where S: std::borrow::Borrow<str> {
    type Output = Number;

    fn index(&self, index: S) -> &Self::Output {
        &self.0[index.borrow()]
    }
}
