use crate::storage::StorageError;

/// Error enum.
#[derive(Debug)]
pub enum Error {
    Parse,
    Storage(StorageError),
}
