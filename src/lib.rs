//! Russian morphological analysis and inflection with compiled OpenCorpora dictionaries.
//!
//! The Russian dictionary is embedded in the binary; an external dictionary can also be loaded.
//! Word/tag frequencies rank results; equal frequencies retain stable heuristic ordering.
//! Analyzer clones share the dictionary, frequency model and a bounded result cache.
//!
//! ```
//! use rs_morphy4::MorphAnalyzer;
//!
//! let morph = MorphAnalyzer::default();
//! let parses = morph.parse("кошки")?;
//! assert_eq!(parses[0].normal_form, "кошка");
//! let form = morph.inflect(&parses[0], ["plur", "gent"])?;
//! assert_eq!(form.map(|p| p.word).as_deref(), Some("кошек"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Scores are rule weights, not probabilities. Synthetic analyses may have no
//! paradigm; their lexemes then contain only the supplied form.

/// Word analysis, ranking, shared caches and inflection.
pub mod analysis;
/// Compiled dictionary tables, binary readers and typed loading errors.
pub mod dictionary;
mod tag;

pub use tag::Tag;

pub use analysis::{CacheInfo, DictionaryAnalyzer, InflectionError, MorphAnalyzer, Parse};

pub use dictionary::{DictionaryData, WordTagProbabilities};
