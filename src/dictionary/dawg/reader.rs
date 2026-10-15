use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::dictionary::dawg::storage::Storage;
use crate::dictionary::error::DictionaryError;
use crate::dictionary::file;

use crate::dictionary::dawg::records::DawgRecord;
use crate::dictionary::dawg::spelling::SpellingMatch;

#[derive(Debug)]
/// In-memory DAWG with typed records and checked traversal.
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
        Self::from_bytes(&bytes, path)
    }

    /// Decodes an in-memory DAWG; the supplied path labels errors and is never opened.
    /// Traversed edges and payloads are checked during lookup.
    pub fn from_bytes(
        bytes: &[u8],
        source_path: impl AsRef<Path>,
    ) -> Result<Self, DictionaryError> {
        let path = source_path.as_ref();
        let storage =
            Storage::decode(bytes).map_err(|source| DictionaryError::binary(path, source))?;
        Ok(Self {
            path: path.to_owned(),
            storage,
            record: PhantomData,
        })
    }

    /// Returns dictionary spellings with independent е -> ё substitutions.
    /// Explicit ё is preserved. Only existing DAWG branches are traversed.
    pub fn get_with_yo(&self, key: &str) -> Result<Vec<SpellingMatch<R>>, DictionaryError> {
        if !key.contains('е') {
            let records = self.get(key)?;
            return Ok(if records.is_empty() {
                Vec::new()
            } else {
                vec![SpellingMatch {
                    word: key.to_owned(),
                    records,
                }]
            });
        }
        self.storage
            .lookup_spellings(key)
            .map_err(|source| DictionaryError::binary(&self.path, source))
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

impl RecordDawg<crate::dictionary::dawg::records::WordRecord> {
    /// Decodes the bundled word DAWG without filesystem access.
    pub fn embedded() -> Result<Self, DictionaryError> {
        Self::from_bytes(crate::dictionary::embedded::WORDS, "<embedded>/words.dawg")
    }
}
