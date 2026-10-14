use crate::dictionary::binary::error::BinaryError;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// A dictionary file could not be read, decoded or validated.
#[derive(Debug)]
pub enum DictionaryError {
    /// A binary structure or traversed record is invalid.
    BinaryError {
        /// Dictionary file involved in the failure.
        path: PathBuf,
        /// Underlying decoding or I/O error.
        source: BinaryError,
    },
    /// A dictionary file could not be read.
    IoError {
        /// Dictionary file involved in the failure.
        path: PathBuf,
        /// Underlying decoding or I/O error.
        source: std::io::Error,
    },
    /// JSON syntax or a serialized field type is invalid.
    ParseError {
        /// Dictionary file involved in the failure.
        path: PathBuf,
        /// Underlying decoding or I/O error.
        source: serde_json::Error,
    },
    /// Decoded content violates a dictionary constraint.
    InvalidData {
        /// Dictionary file involved in the failure.
        path: PathBuf,
        /// Violated dictionary constraint.
        reason: ValidationError,
    },
}

/// A violated constraint in a dictionary file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Metadata has no filename for the required tag format.
    MissingTable {
        /// Required tag notation.
        format: String,
    },
    /// A table name is not a single filename inside the dictionary directory.
    InvalidTableFilename {
        /// Rejected table filename.
        filename: String,
    },
    /// A feature definition has an empty name.
    EmptyGrammemeName,
    /// Multiple feature definitions share a name.
    DuplicateGrammemeName {
        /// Feature code.
        name: String,
    },
    /// A feature names an undefined parent category.
    UnknownParent {
        /// Feature code.
        name: String,
        /// Undefined parent feature code.
        parent: String,
    },
    /// A tag contains an undefined or empty feature code.
    UnknownGrammeme {
        /// Feature code.
        name: String,
        /// Tag table index.
        tag_index: usize,
    },
    /// A metadata key occurs more than once.
    DuplicateMetadataKey {
        /// Duplicated metadata key.
        key: String,
    },
    /// The compiled format version is unsupported.
    UnsupportedFormat {
        /// Value recorded in the dictionary.
        found: String,
        /// Required value or count.
        expected: String,
    },
    /// The dictionary language is unsupported.
    UnsupportedLanguage {
        /// Value recorded in the dictionary.
        found: String,
        /// Required value or count.
        expected: String,
    },
    /// The prefix table is empty.
    EmptyParadigmPrefixes,
    /// A JSON table length differs from metadata.
    LengthMismatch {
        /// Required value or count.
        expected: usize,
        /// Observed count.
        actual: usize,
    },
}

impl DictionaryError {
    /// Associates a binary decoding error with its dictionary file.
    pub fn binary(path: &Path, source: BinaryError) -> Self {
        Self::BinaryError {
            path: path.to_owned(),
            source,
        }
    }

    /// Associates a validation failure with its dictionary file.
    pub fn invalid(path: &Path, reason: ValidationError) -> Self {
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
