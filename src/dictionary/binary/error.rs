use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Dictionary table targeted by a binary reference.
pub enum TableKind {
    /// Suffix string table.
    Suffix,
    /// Grammatical tag table.
    Tag,
    /// Paradigm prefix table.
    Prefix,
    /// Inflection paradigm table.
    Paradigm,
    /// Form within a paradigm.
    Form,
}

/// A binary structure or record violates the dictionary format.
#[derive(Debug)]
pub enum BinaryError {
    /// A binary read extends beyond the available bytes.
    UnexpectedEnd {
        /// Byte offset of the failing read.
        offset: usize,
        /// Number of bytes requested.
        needed: usize,
        /// Number of bytes remaining.
        remaining: usize,
    },
    /// Unexpected bytes follow the decoded structure.
    TrailingData {
        /// Byte offset of the failing read.
        offset: usize,
        /// Number of bytes remaining.
        remaining: usize,
    },
    /// The binary count differs from metadata.
    CountMismatch {
        /// Required value or count.
        expected: usize,
        /// Observed count.
        actual: usize,
    },
    /// A paradigm is empty or cannot be split into three parallel arrays.
    InvalidParadigmLength {
        /// Paradigm identifier.
        paradigm: usize,
        /// Number of entries in the referenced array.
        length: usize,
    },
    /// A paradigm form references a nonexistent table entry.
    InvalidReference {
        /// Table containing the referenced entry.
        table: TableKind,
        /// Invalid entry identifier.
        index: usize,
        /// Number of entries in the referenced array.
        length: usize,
        /// Paradigm identifier.
        paradigm: usize,
        /// Form identifier within the paradigm.
        form: usize,
    },
    /// The DAWG has no root unit.
    EmptyDawg,
    /// Traversal references a unit outside the DAWG array.
    UnitOutOfBounds {
        /// Invalid entry identifier.
        index: usize,
        /// Number of entries in the referenced array.
        length: usize,
    },
    /// A guide edge has no matching DAWG transition.
    InvalidTransition {
        /// Invalid entry identifier.
        index: usize,
        /// Transition byte.
        label: u8,
    },
    /// A guide repeats an already visited sibling label.
    RepeatedSibling {
        /// Invalid entry identifier.
        index: usize,
        /// Transition byte.
        label: u8,
    },
    /// A payload path ends without a terminal unit.
    MissingTerminal {
        /// Invalid entry identifier.
        index: usize,
    },
    /// A terminal points to a unit that does not hold a value.
    InvalidLeaf {
        /// Invalid entry identifier.
        index: usize,
    },
    /// The encoded payload exceeds the supported record length.
    PayloadTooLong {
        /// Maximum encoded payload size in bytes.
        limit: usize,
    },
    /// The record payload is not valid base64.
    InvalidBase64 {
        /// Underlying decoding or I/O error.
        source: base64::DecodeError,
    },
    /// The decoded payload has the wrong byte count.
    InvalidRecordLength {
        /// Required value or count.
        expected: usize,
        /// Observed count.
        actual: usize,
    },
    /// The word does not have its recorded paradigm affixes.
    AffixMismatch {
        /// Paradigm identifier.
        paradigm: usize,
        /// Form identifier within the paradigm.
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
