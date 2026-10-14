use std::path::{Path, PathBuf};

use crate::dictionary::dawg::{storage::Storage, units};
use crate::dictionary::error::DictionaryError;

/// Integer completion DAWG with checked exact-key lookup.
#[derive(Debug)]
pub struct IntegerDawg {
    storage: Storage,
    path: PathBuf,
}

impl IntegerDawg {
    /// Decodes units and guide arrays; traversed transitions are checked during lookup.
    pub fn from_bytes(
        bytes: &[u8],
        source_path: impl AsRef<Path>,
    ) -> Result<Self, DictionaryError> {
        let path = source_path.as_ref();
        Ok(Self {
            storage: Storage::decode(bytes)
                .map_err(|source| DictionaryError::binary(path, source))?,
            path: path.to_owned(),
        })
    }

    /// Returns the stored integer, including zero, or None for a missing key.
    pub fn get(&self, key: &str) -> Result<Option<u32>, DictionaryError> {
        if key.bytes().any(|byte| byte == 0) {
            return Ok(None);
        }
        let lookup = || {
            let mut index = 0;
            for label in key.bytes() {
                match units::follow(&self.storage.units, index, label)? {
                    Some(next) => index = next,
                    None => return Ok(None),
                }
            }
            if !units::has_leaf(units::unit(&self.storage.units, index)?) {
                return Ok(None);
            }
            units::value(&self.storage.units, index).map(Some)
        };
        lookup().map_err(|source| DictionaryError::binary(&self.path, source))
    }
}
