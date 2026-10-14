use std::collections::BTreeMap;

use serde::Deserialize;

/// Compilation settings recorded in the dictionary metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct CompileOptions {
    /// Minimum ending frequency used by the compiler.
    pub min_ending_freq: usize,
    /// Minimum paradigm frequency used by the compiler.
    pub min_paradigm_popularity: usize,
    /// Maximum prediction suffix length in characters.
    pub max_suffix_length: usize,
    /// Prefix strings indexed by binary paradigm forms.
    pub paradigm_prefixes: Vec<String>,
}

/// Metadata for compiled dictionary format 2.4.
#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    /// Dictionary language code.
    pub language_code: String,
    /// Compiled file format version.
    pub format_version: String,
    /// Compiler version recorded by the source dictionary.
    pub pymorphy2_version: String,
    /// Compilation timestamp recorded by the source.
    pub compiled_at: String,
    /// Source dictionary name.
    pub source: String,
    /// Source data format version.
    pub source_version: String,
    /// Source data revision.
    pub source_revision: String,
    /// Number of source lexemes before compilation.
    pub source_lexemes_count: usize,
    /// Number of links between source lexemes.
    pub source_links_count: usize,
    /// Number of compiled tags.
    pub gramtab_length: usize,
    /// Tag notation names mapped to table filenames.
    pub gramtab_formats: BTreeMap<String, String>,
    /// Number of inflection paradigms.
    pub paradigms_length: usize,
    /// Number of distinct suffixes.
    pub suffixes_length: usize,
    /// Number of compiled word records, not bytes.
    pub words_dawg_length: usize,
    /// Settings used to compile the dictionary.
    pub compile_options: CompileOptions,
    /// Prediction record counts for each paradigm prefix.
    pub prediction_suffixes_dawg_lengths: Vec<usize>,
}

/// A grammatical feature and its optional parent category.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "[String; 4]")]
pub struct Grammeme {
    /// Canonical feature code.
    pub name: String,
    /// An empty string denotes a root category.
    pub parent: String,
    /// Human-readable feature abbreviation.
    pub alias: String,
    /// Feature description supplied by the dictionary.
    pub description: String,
}

impl From<[String; 4]> for Grammeme {
    fn from([name, parent, alias, description]: [String; 4]) -> Self {
        Self {
            name,
            parent,
            alias,
            description,
        }
    }
}
