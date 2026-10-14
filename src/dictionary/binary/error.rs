use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Suffix,
    Tag,
    Prefix,
    Paradigm,
    Form,
}

/// A binary structure or record violates the dictionary format.
#[derive(Debug)]
pub enum BinaryError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    TrailingData {
        offset: usize,
        remaining: usize,
    },
    CountMismatch {
        expected: usize,
        actual: usize,
    },
    InvalidParadigmLength {
        paradigm: usize,
        length: usize,
    },
    InvalidReference {
        table: TableKind,
        index: usize,
        length: usize,
        paradigm: usize,
        form: usize,
    },
    EmptyDawg,
    UnitOutOfBounds {
        index: usize,
        length: usize,
    },
    InvalidTransition {
        index: usize,
        label: u8,
    },
    RepeatedSibling {
        index: usize,
        label: u8,
    },
    MissingTerminal {
        index: usize,
    },
    InvalidLeaf {
        index: usize,
    },
    PayloadTooLong {
        limit: usize,
    },
    InvalidBase64 {
        source: base64::DecodeError,
    },
    InvalidRecordLength {
        expected: usize,
        actual: usize,
    },
    AffixMismatch {
        paradigm: usize,
        form: usize,
    },
}

impl fmt::Display for TableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Suffix => "suffix",
            Self::Tag => "tag",
            Self::Prefix => "prefix",
            Self::Paradigm => "paradigm",
            Self::Form => "form",
        })
    }
}

impl fmt::Display for BinaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                remaining,
            } => write!(
                f,
                "at byte {offset}: need {needed} bytes, only {remaining} remain"
            ),
            Self::TrailingData { offset, remaining } => {
                write!(f, "{remaining} unexpected bytes after byte {offset}")
            }
            Self::CountMismatch { expected, actual } => {
                write!(f, "expected {expected} entries, found {actual}")
            }
            Self::InvalidParadigmLength { paradigm, length } => {
                write!(f, "paradigm {paradigm} has invalid array length {length}")
            }
            Self::InvalidReference {
                table,
                index,
                length,
                paradigm,
                form,
            } => write!(
                f,
                "paradigm {paradigm}, form {form}: {table} index {index} is outside {length} entries"
            ),
            Self::EmptyDawg => f.write_str("DAWG contains no root unit"),
            Self::UnitOutOfBounds { index, length } => {
                write!(f, "DAWG unit {index} is outside {length} units")
            }
            Self::InvalidTransition { index, label } => write!(
                f,
                "DAWG guide at unit {index} references missing transition {label}"
            ),
            Self::RepeatedSibling { index, label } => write!(
                f,
                "DAWG guide at unit {index} repeats sibling label {label}"
            ),
            Self::MissingTerminal { index } => {
                write!(f, "DAWG path ends at nonterminal unit {index}")
            }
            Self::InvalidLeaf { index } => {
                write!(f, "DAWG unit {index} references a non-leaf value")
            }
            Self::PayloadTooLong { limit } => {
                write!(f, "DAWG payload exceeds {limit} encoded bytes")
            }
            Self::InvalidBase64 { source } => write!(f, "invalid DAWG payload encoding: {source}"),
            Self::InvalidRecordLength { expected, actual } => {
                write!(f, "expected {expected} record bytes, found {actual}")
            }
            Self::AffixMismatch { paradigm, form } => write!(
                f,
                "word does not match the affixes of paradigm {paradigm}, form {form}"
            ),
        }
    }
}

impl Error for BinaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidBase64 { source } => Some(source),
            _ => None,
        }
    }
}
