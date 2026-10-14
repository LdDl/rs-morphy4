use std::path::Path;

/// A dictionary file could not be read, decoded or validated.
#[derive(Debug, Clone)]
pub enum DictionaryError {
    /// A file could not be read.
    IoError(String),
    /// JSON could not be decoded into the expected types.
    ParseError(String),
    /// Dictionary contents failed validation.
    InvalidData(String),
}

impl DictionaryError {
    pub(crate) fn invalid(path: &Path, reason: impl Into<String>) -> Self {
        Self::InvalidData(format!(
            "invalid dictionary file {}: {}",
            path.display(),
            reason.into()
        ))
    }
}
