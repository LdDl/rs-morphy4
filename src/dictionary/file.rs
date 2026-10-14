use std::path::Path;

use crate::dictionary::error::DictionaryError;

pub fn read(path: &Path) -> Result<Vec<u8>, DictionaryError> {
    std::fs::read(path).map_err(|source| DictionaryError::IoError {
        path: path.to_owned(),
        source,
    })
}
