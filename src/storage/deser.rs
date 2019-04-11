//! Record deserialization.

use serde::{
    de::{
        value::BorrowedStrDeserializer,
        DeserializeSeed,
        Error as SerdeError,
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
                    Err(SerdeError::custom(DeserError::Parse))
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
                E: SerdeError,
            {
                Ok(Number::Int(i64::from(int)))
            }

            fn visit_i32<E>(self, int: i32) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Int(i64::from(int)))
            }

            fn visit_i64<E>(self, int: i64) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Int(int))
            }

            fn visit_u8<E>(self, uint: u8) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Int(i64::from(uint)))
            }

            fn visit_u32<E>(self, uint: u32) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Int(i64::from(uint)))
            }

            fn visit_u64<E>(self, uint: u64) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Int(uint as i64))
            }

            fn visit_f32<E>(self, float: f32) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Float(f64::from(float)))
            }

            fn visit_f64<E>(self, float: f64) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Number::Float(float))
            }

            fn visit_borrowed_str<E>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Self::parse_str(v)
                    .map_err(|_| SerdeError::custom(DeserError::Parse))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Self::parse_str(v.as_str())
                    .map_err(|_| SerdeError::custom(DeserError::Parse))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Self::parse_str(v)
                    .map_err(|_| SerdeError::custom(DeserError::Parse))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                let d = v
                    .to_digit(10)
                    .ok_or_else(|| SerdeError::custom(DeserError::Parse))?;

                Ok(Number::Int(i64::from(d)))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                let s = std::str::from_utf8(v).map_err(SerdeError::custom)?;

                Self::parse_str(s)
                    .map_err(|_| SerdeError::custom(DeserError::Parse))
            }

            fn visit_borrowed_bytes<E>(
                self,
                v: &'de [u8],
            ) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                let s = std::str::from_utf8(v)
                    .map_err(|_| SerdeError::custom(DeserError::Parse))?;

                Self::parse_str(s).map_err(SerdeError::custom)
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                let s = std::str::from_utf8(&v)
                    .map_err(|_| SerdeError::custom(DeserError::Parse))?;

                Self::parse_str(s).map_err(SerdeError::custom)
            }
        }

        deserializer.deserialize_f64(NumberVisitor {})
    }
}

/// A newtype containing field's name - either a [`String`] or a [`&str`][str].
pub struct Field<'a>(Cow<'a, str>);

impl<'a, 'de: 'a> Deserialize<'de> for Field<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor<'a> {
            lifetime: PhantomData<Field<'a>>,
        }

        impl<'a, 'de: 'a> Visitor<'de> for FieldVisitor<'a> {
            type Value = Field<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a number, string-like type or bytes a number can be parsed from")
            }

            fn visit_borrowed_str<E>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Borrowed(v)))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Owned(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Owned(v.to_owned())))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Owned(
                    String::from_utf8(v.to_vec())
                        .map_err(SerdeError::custom)?,
                )))
            }

            fn visit_borrowed_bytes<E>(
                self,
                v: &'de [u8],
            ) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Borrowed(
                    std::str::from_utf8(v).map_err(SerdeError::custom)?,
                )))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(Field(Cow::Owned(
                    String::from_utf8(v).map_err(SerdeError::custom)?,
                )))
            }
        }

        deserializer.deserialize_str(FieldVisitor {
            lifetime: PhantomData,
        })
    }
}

impl<'de, 'a> DeserializeSeed<'de> for &'a mut Storage {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendStorageVisitor<'a>(&'a mut Storage);

        impl<'de, 'a> Visitor<'de> for ExtendStorageVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map of numbers with strings as keys")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let record = (0..)
                    .map(|_| map.next_entry::<Field, Number>())
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
                    .collect::<Result<HashMap<_, _, _>, _>>()?;

                self.0.push_record(record).map_err(SerdeError::custom)
            }
        }

        deserializer.deserialize_map(ExtendStorageVisitor(self))
    }
}

/// A newtype containing two iterators - one over keys and one over values.
///
/// This type is useful because it implements [`MapAccess`] using [`BorrowedStrDeserializer`]
/// so it doesn't require creating a new [`String`] to deserialize to [`&str`][str].
pub struct Record<'a, K, V>(pub K, pub V)
where
    K: Iterator<Item = &'a str>,
    V: Iterator<Item = &'a str>;

impl<'de, 'a: 'de, K, V> MapAccess<'de> for Record<'a, K, V>
where
    K: Iterator<Item = &'a str>,
    V: Iterator<Item = &'a str>,
{
    type Error = ::serde::de::value::Error;

    fn next_key_seed<SK>(
        &mut self,
        seed: SK,
    ) -> Result<Option<SK::Value>, Self::Error>
    where
        SK: DeserializeSeed<'de>,
    {
        let key = self.0.next();

        match key {
            None => Ok(None),
            Some(key) => seed
                .deserialize::<BorrowedStrDeserializer<::serde::de::value::Error>>(BorrowedStrDeserializer::new(key))
                .map(Some)
                .map_err(SerdeError::custom),
        }
    }

    fn next_value_seed<SV>(
        &mut self,
        seed: SV,
    ) -> Result<SV::Value, Self::Error>
    where
        SV: DeserializeSeed<'de>,
    {
        let value = self
            .1
            .next()
            .ok_or_else(|| SerdeError::custom(DeserError::NextOnEmptyIter))?;

        seed.deserialize::<BorrowedStrDeserializer<::serde::de::value::Error>>(
            BorrowedStrDeserializer::new(value),
        )
        .map_err(SerdeError::custom)
    }
}
