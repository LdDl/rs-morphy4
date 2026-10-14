use super::BinaryError;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// A dictionary file could not be read, decoded or validated.
#[derive(Debug)]
pub enum DictionaryError {
    BinaryError { path: PathBuf, source: BinaryError },
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseError {
        path: PathBuf,
        source: serde_json::Error,
    },
    InvalidData {
        path: PathBuf,
        reason: ValidationError,
    },
}

/// A violated constraint in a dictionary file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    MissingTable { format: String },
    InvalidTableFilename { filename: String },
    EmptyGrammemeName,
    DuplicateGrammemeName { name: String },
    UnknownParent { name: String, parent: String },
    UnknownGrammeme { name: String, tag_index: usize },
    DuplicateMetadataKey { key: String },
    UnsupportedFormat { found: String, expected: String },
    UnsupportedLanguage { found: String, expected: String },
    EmptyParadigmPrefixes,
    LengthMismatch { expected: usize, actual: usize },
}

impl DictionaryError {
    pub(crate) fn binary(path: &Path, source: BinaryError) -> Self {
        Self::BinaryError { path: path.to_owned(), source }
    }

    pub(crate) fn invalid(path: &Path, reason: ValidationError) -> Self {
        Self::InvalidData {
            path: path.to_owned(),
            reason,
        }
    }
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError { path, source } => {
                write!(
                    f,
                    "cannot read dictionary file {}: {source}",
                    path.display()
                )
            }
            Self::ParseError { path, source } => {
                write!(
                    f,
                    "cannot parse dictionary file {}: {source}",
                    path.display()
                )
            }
            Self::BinaryError { path, source } => {
                write!(f, "invalid binary dictionary {}: {source}", path.display())
            }
            Self::InvalidData { path, reason } => {
                write!(f, "invalid dictionary file {}: {reason}", path.display())
            }
        }
    }
}

impl Error for DictionaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BinaryError { source, .. } => Some(source),
            Self::IoError { source, .. } => Some(source),
            Self::ParseError { source, .. } => Some(source),
            Self::InvalidData { reason, .. } => Some(reason),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTable { format } => {
                write!(f, "missing {format} table filename")
            }
            Self::InvalidTableFilename { filename } => {
                write!(
                    f,
                    "table path {filename:?} must be a filename within the dictionary directory"
                )
            }
            Self::EmptyGrammemeName => f.write_str("empty grammeme name"),
            Self::DuplicateGrammemeName { name } => {
                write!(f, "duplicate grammeme name {name:?}")
            }
            Self::UnknownParent { name, parent } => {
                write!(f, "unknown parent {parent:?} of grammeme {name:?}")
            }
            Self::UnknownGrammeme { name, tag_index } => {
                write!(f, "unknown or empty grammeme {name:?} in tag {tag_index}")
            }
            Self::DuplicateMetadataKey { key } => {
                write!(f, "duplicate metadata key {key:?}")
            }
            Self::UnsupportedFormat { found, expected } => {
                write!(f, "unsupported format {found:?}; expected {expected}")
            }
            Self::UnsupportedLanguage { found, expected } => {
                write!(f, "unsupported language {found:?}; expected {expected}")
            }
            Self::EmptyParadigmPrefixes => f.write_str("paradigm prefixes are empty"),
            Self::LengthMismatch { expected, actual } => {
                write!(f, "expected {expected} entries, found {actual}")
            }
        }
    }
}

impl Error for ValidationError {}
