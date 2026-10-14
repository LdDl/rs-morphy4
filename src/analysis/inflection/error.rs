use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflectionError {
    IncompleteReference,
    InvalidParadigm {
        index: u16,
        length: usize,
    },
    InvalidForm {
        paradigm: u16,
        index: u16,
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
