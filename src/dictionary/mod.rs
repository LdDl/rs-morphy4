mod binary;
mod data;
pub mod dawg;
mod embedded;
mod error;
mod file;
mod metadata;
mod paradigm;
mod source;
mod tables;

pub use binary::error::{BinaryError, TableKind};
pub use error::{DictionaryError, ValidationError};
pub use metadata::{CompileOptions, Grammeme, Metadata};
pub use paradigm::{Paradigm, ParadigmForm, Paradigms};
pub use tables::DictionaryTables;

pub use data::DictionaryData;

mod probabilities;
mod probability_tags;
pub use probabilities::WordTagProbabilities;
