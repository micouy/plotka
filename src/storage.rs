//! Data storage.

use std::{borrow::Cow, collections::HashMap, fmt};

use crate::Record;

/// Either `Vec<f64>` or `Vec<i64>`. Used as a "column" in [`Storage`].
#[derive(Debug)]
pub enum NumberVec {
    /// A vector of floats.
    Float(Vec<f64>),
    /// A vector of integers.
    Int(Vec<i64>),
}

impl NumberVec {
    /// Construct the vector from [`Number`].
    pub fn from_number(number: Number) -> Self {
        match number {
            Number::Float(number) => NumberVec::Float(vec![number]),
            Number::Int(number) => NumberVec::Int(vec![number]),
        }
    }

    /// Get vector's length.
    pub fn len(&self) -> usize {
        match self {
            NumberVec::Float(vec) => vec.len(),
            NumberVec::Int(vec) => vec.len(),
        }
    }

    /// Check whether the vector is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            NumberVec::Float(vec) => vec.is_empty(),
            NumberVec::Int(vec) => vec.is_empty(),
        }
    }

    /// Get a reference to the inner vector of floats. Returns `None` if it's
    /// not a vector of floats.
    pub fn float(&self) -> Option<&Vec<f64>> {
        match self {
            NumberVec::Float(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get a mutable reference to the inner vector of floats. Returns `None` if
    /// it's not a vector of floats.
    pub fn float_mut(&mut self) -> Option<&mut Vec<f64>> {
        match self {
            NumberVec::Float(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get a reference to the inner vector of floats. Returns `None` if it's
    /// not a vector of integers.
    pub fn int(&self) -> Option<&Vec<i64>> {
        match self {
            NumberVec::Int(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get a mutable reference to the inner vector of floats. Returns `None` if
    /// it's not a vector of integers.
    pub fn int_mut(&mut self) -> Option<&mut Vec<i64>> {
        match self {
            NumberVec::Int(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get an element by index. Returns [`Number`] which contains either a
    /// float or an integer, depending on the type of the vector.
    pub fn get(&self, index: usize) -> Option<Number> {
        match self {
            NumberVec::Float(vec) =>
                vec.get(index).map(|number| Number::Float(*number)),
            NumberVec::Int(vec) =>
                vec.get(index).map(|number| Number::Int(*number)),
        }
    }
}

/// A number - either a float or an integer.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// A float.
    Float(f64),
    /// An integer.
    Int(i64),
}

impl From<f64> for Number {
    fn from(number: f64) -> Self {
        Number::Float(number)
    }
}

impl From<i64> for Number {
    fn from(number: i64) -> Self {
        Number::Int(number)
    }
}

/// Storage error.
#[derive(Debug)]
pub enum StorageError {
    /// Fields of the map deserialized from the input do not match the fields
    /// in the record storage.
    FieldMismatch,
    /// Number's type (derived from whether it can be parsed to an integer or
    /// not) does not match the type of storage's field.
    FieldTypeMismatch,

    /// Received an empty record.
    EmptyRecord,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[allow(unreachable_patterns)]
        match self {
            StorageError::FieldMismatch => write!(f, "field mismatch"),
            StorageError::FieldTypeMismatch => write!(f, "field type mismatch"),
            _ => write!(f, "storage error"),
        }
    }
}

/// Record storage.
#[derive(Debug)]
pub struct Storage {
    inner: HashMap<String, NumberVec>,
    is_empty: bool,
}

impl Storage {
    /// Construct new storage.
    pub fn new() -> Self {
        Storage {
            inner: HashMap::new(),
            is_empty: true,
        }
    }

    fn push_record_first<'a>(
        &mut self,
        record: Record<'a>,
    ) -> Result<(), StorageError> {
        let record = record.0;

        self.inner = record
            .into_iter()
            .map(|(key, number)| {
                (key.into_owned(), NumberVec::from_number(number))
            })
            .collect();

        if self.inner.is_empty() {
            return Err(StorageError::EmptyRecord);
        }

        self.is_empty = false;

        Ok(())
    }

    fn push_record_next<'a>(
        &mut self,
        record: Record<'a>,
    ) -> Result<(), StorageError> {
        let record = record.0;

        fn keys_match(
            inner: &HashMap<String, NumberVec>,
            record: &HashMap<Cow<str>, Number>,
        ) -> bool {
            inner.keys().all(|key| record.contains_key(key.as_str()))
        }

        if record.len() != self.inner.len() || !keys_match(&self.inner, &record)
        {
            return Err(StorageError::FieldMismatch);
        }

        let types_match = record.iter().all(|(key, number)| {
            let vec = self.inner.get_mut(&**key).unwrap();

            match (vec, number) {
                (NumberVec::Int(_), Number::Int(_)) => true,
                (NumberVec::Float(_), Number::Float(_)) => true,
                _ => false,
            }
        });

        if !types_match {
            Err(StorageError::FieldTypeMismatch)
        } else {
            record.into_iter().for_each(|(key, number)| {
                let vec = self.inner.get_mut(&*key).unwrap();

                match (vec, number) {
                    (NumberVec::Int(vec), Number::Int(int)) => vec.push(int),
                    (NumberVec::Float(vec), Number::Float(float)) =>
                        vec.push(float),
                    _ => unreachable!(),
                }
            });

            Ok(())
        }
    }

    /// Push a record to the storage.
    pub fn push_record<'a>(
        &mut self,
        record: Record<'a>,
    ) -> Result<(), StorageError> {
        if !self.is_empty {
            self.push_record_next(record)
        } else {
            self.push_record_first(record)
        }
    }
}

impl<S> std::ops::Index<S> for Storage
where
    S: std::borrow::Borrow<str>,
{
    type Output = NumberVec;

    fn index(&self, field: S) -> &Self::Output {
        &self.inner[field.borrow()]
    }
}
