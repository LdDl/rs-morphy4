use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::dictionary::dawg::storage::Storage;
use crate::dictionary::error::DictionaryError;
use crate::dictionary::file;

use crate::dictionary::dawg::records::DawgRecord;

#[derive(Debug)]
pub struct RecordDawg<R: DawgRecord> {
    path: PathBuf,
    storage: Storage,
    record: PhantomData<R>,
}

impl<R: DawgRecord> RecordDawg<R> {
    /// Validates array sizes at load time; traversed edges and payloads are checked during lookup.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        let path = path.as_ref();
        let bytes = file::read(path)?;
        let storage =
            Storage::decode(&bytes).map_err(|source| DictionaryError::binary(path, source))?;
        Ok(Self {
            path: path.to_owned(),
            storage,
            record: PhantomData,
        })
    }

    /// Returns every record for an exact, case-sensitive key in stored guide order.
    ///
    /// A missing key produces an empty vector. Invalid data produces an error.
    pub fn get(&self, key: &str) -> Result<Vec<R>, DictionaryError> {
        self.storage
            .lookup(key)
            .map_err(|source| DictionaryError::binary(&self.path, source))
    }
}
