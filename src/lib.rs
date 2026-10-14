//! Russian morphological analysis and inflection with compiled OpenCorpora dictionaries.
//!
//! The dictionary is loaded from an explicit directory and retained in memory.
//! Analyzer clones share both the dictionary and a bounded result cache.
//!
//! ```
//! use rs_morphy4::MorphAnalyzer;
//!
//! let morph = MorphAnalyzer::load("data/dicts_ru")?;
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
