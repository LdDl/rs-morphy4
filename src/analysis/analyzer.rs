use std::path::Path;

use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::normalization;
use crate::analysis::parse::Parse;
use crate::analysis::word;
use crate::dictionary::DictionaryError;

/// Dictionary, token, prediction and compound rules without result ranking.
#[derive(Debug)]
pub struct MorphAnalyzer {
    dictionary: DictionaryAnalyzer,
}

impl MorphAnalyzer {
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        Ok(Self {
            dictionary: DictionaryAnalyzer::load(directory)?,
        })
    }

    pub fn dictionary(&self) -> &DictionaryAnalyzer {
        &self.dictionary
    }

    /// Returns analyses in rule order. Scores are rule weights, not probabilities.
    pub fn parse(&self, input: &str) -> Result<Vec<Parse>, DictionaryError> {
        normalization::parse(input, |input| word::parse(&self.dictionary, input))
    }
}
