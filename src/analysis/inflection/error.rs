use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A parse has incomplete or invalid metadata for word-form reconstruction.
pub enum InflectionError {
    /// Paradigm, stem and form metadata are inconsistent.
    IncompleteReference,
    /// The paradigm identifier is outside the loaded table.
    InvalidParadigm {
        /// Invalid entry identifier.
        index: u16,
        /// Number of entries in the referenced array.
        length: usize,
    },
    /// The form identifier is outside its paradigm.
    InvalidForm {
        /// Paradigm identifier.
        paradigm: u16,
        /// Invalid entry identifier.
        index: u16,
        /// Number of entries in the referenced array.
        length: usize,
    },
}

impl fmt::Display for InflectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteReference => {
                f.write_str("inflection requires both paradigm and stem, or neither")
            }
            Self::InvalidParadigm { index, length } => {
                write!(f, "paradigm {index} is outside {length} entries")
            }
            Self::InvalidForm {
                paradigm,
                index,
                length,
            } => write!(
                f,
                "form {index} is outside the {length} forms of paradigm {paradigm}"
            ),
        }
    }
}

impl Error for InflectionError {}
