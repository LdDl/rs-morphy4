use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::normalization;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

impl DictionaryAnalyzer {
    /// Normalizes case and whitespace and resolves dictionary е -> ё variants.
    /// Explicit ё is preserved. No prediction or ranking is performed.
    pub fn parse(&self, word: &str) -> Result<Vec<Parse>, DictionaryError> {
        normalization::parse(word, |word| self.parse_with_yo(word))
    }
}
