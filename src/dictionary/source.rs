use std::borrow::Cow;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::dictionary::data::DictionaryData;
use crate::dictionary::error::DictionaryError;
use crate::dictionary::file;

pub enum Source<'a> {
    Directory(&'a Path),
    Memory(DictionaryData<'a>),
}

pub enum JsonTable {
    Metadata,
    Grammemes,
    Tags,
    Suffixes,
}

impl Source<'_> {
    pub fn path(&self, name: &str) -> PathBuf {
        match self {
            Self::Directory(directory) => directory.join(name),
            Self::Memory(_) => Path::new("<memory>").join(name),
        }
    }

    pub fn json<T: DeserializeOwned>(
        &self,
        path: &Path,
        table: JsonTable,
    ) -> Result<T, DictionaryError> {
        let bytes = self.read(path, table)?;
        serde_json::from_slice(&bytes).map_err(|source| DictionaryError::ParseError {
            path: path.to_owned(),
            source,
        })
    }

    fn read(&self, path: &Path, table: JsonTable) -> Result<Cow<'_, [u8]>, DictionaryError> {
        match self {
            Self::Directory(_) => file::read(path).map(Cow::Owned),
            Self::Memory(data) => Ok(Cow::Borrowed(match table {
                JsonTable::Metadata => data.metadata,
                JsonTable::Grammemes => data.grammemes,
                JsonTable::Tags => data.tags,
                JsonTable::Suffixes => data.suffixes,
            })),
        }
    }
}
