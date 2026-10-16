use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::dictionary::dawg::{PredictionRecord, RecordDawg};
use crate::dictionary::error::{DictionaryError, ValidationError};

/// Suffix records for one paradigm prefix, with their diagnostic source path.
#[derive(Debug)]
pub struct SuffixPrediction {
    /// Decoded suffix table.
    pub records: RecordDawg<PredictionRecord>,
    /// Source label used for invalid paradigm references.
    pub path: PathBuf,
}

impl SuffixPrediction {
    /// Decodes a complete set of tables; None omits compiled predictions.
    pub fn from_data(
        data: Option<&[&[u8]]>,
        prefix_count: usize,
    ) -> Result<Vec<Self>, DictionaryError> {
        let Some(data) = data else {
            return Ok(Vec::new());
        };
        if data.len() != prefix_count {
            return Err(DictionaryError::invalid(
                Path::new("<memory>/prediction-suffixes"),
                ValidationError::LengthMismatch {
                    expected: prefix_count,
                    actual: data.len(),
                },
            ));
        }
        data.iter()
            .enumerate()
            .map(|(index, bytes)| {
                let path = PathBuf::from(format!("<memory>/prediction-suffixes-{index}.dawg"));
                Ok(Self {
                    records: RecordDawg::from_bytes(bytes, &path)?,
                    path,
                })
            })
            .collect()
    }

    /// Loads all prefix tables. A wholly absent set omits compiled prediction; a partial set fails.
    pub fn load(directory: &Path, prefix_count: usize) -> Result<Vec<Self>, DictionaryError> {
        let mut result = Vec::new();
        let mut missing = None;
        for index in 0..prefix_count {
            let path = directory.join(format!("prediction-suffixes-{index}.dawg"));
            match RecordDawg::load(&path) {
                Ok(records) => result.push(Self { records, path }),
                Err(DictionaryError::IoError { path, source })
                    if source.kind() == ErrorKind::NotFound =>
                {
                    missing = Some(DictionaryError::IoError { path, source });
                }
                Err(error) => return Err(error),
            }
        }
        if !result.is_empty()
            && let Some(error) = missing
        {
            return Err(error);
        }
        Ok(result)
    }
}
