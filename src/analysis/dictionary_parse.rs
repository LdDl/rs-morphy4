use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::normalization;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

impl DictionaryAnalyzer {
    /// Normalizes case and whitespace and merges the alternate е/ё spelling.
    ///
    /// All е/ё are swapped together for short words or when the spelling is absent.
    /// No prediction or ranking is performed; unknown words return an empty vector.
    pub fn parse(&self, word: &str) -> Result<Vec<Parse>, DictionaryError> {
        normalization::parse(word, |word| self.parse_exact(word))
    }
}
