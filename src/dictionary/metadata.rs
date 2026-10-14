use std::collections::BTreeMap;

use serde::Deserialize;

/// Compilation settings recorded in the dictionary metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct CompileOptions {
    pub min_ending_freq: usize,
    pub min_paradigm_popularity: usize,
    pub max_suffix_length: usize,
    pub paradigm_prefixes: Vec<String>,
}

/// Metadata for compiled dictionary format 2.4.
#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub language_code: String,
    pub format_version: String,
    pub pymorphy2_version: String,
    pub compiled_at: String,
    pub source: String,
    pub source_version: String,
    pub source_revision: String,
    pub source_lexemes_count: usize,
    pub source_links_count: usize,
    pub gramtab_length: usize,
    pub gramtab_formats: BTreeMap<String, String>,
    pub paradigms_length: usize,
    pub suffixes_length: usize,
    pub words_dawg_length: usize,
    pub compile_options: CompileOptions,
    pub prediction_suffixes_dawg_lengths: Vec<usize>,
}

/// A grammatical feature and its optional parent category.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "[String; 4]")]
pub struct Grammeme {
    pub name: String,
    /// An empty string denotes a root category.
    pub parent: String,
    pub alias: String,
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
