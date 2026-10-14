mod binary;
pub mod dawg;
mod error;
mod file;
mod metadata;
mod paradigm;
mod tables;

pub use binary::error::{BinaryError, TableKind};
pub use error::{DictionaryError, ValidationError};
pub use metadata::{CompileOptions, Grammeme, Metadata};
pub use paradigm::{Paradigm, ParadigmForm, Paradigms};
pub use tables::DictionaryTables;
