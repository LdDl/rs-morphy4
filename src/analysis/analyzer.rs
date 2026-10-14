use std::path::Path;
use std::sync::Arc;

use crate::analysis::cache::{CacheInfo, ParseCache};
use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::normalization;
use crate::analysis::parse::Parse;
use crate::analysis::ranking;
use crate::analysis::word;
use crate::dictionary::DictionaryError;

/// Clones share an immutable dictionary and a bounded, thread-safe result cache.
#[derive(Debug, Clone)]
pub struct MorphAnalyzer {
    dictionary: Arc<DictionaryAnalyzer>,
    cache: Arc<ParseCache>,
}

impl MorphAnalyzer {
    /// Loads a dictionary with a FIFO cache of at most 1024 normalized tokens.
    ///
    /// # Errors
    /// Returns an error for unreadable, malformed or unsupported dictionary files.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        Ok(Self::from_shared(
            Arc::new(DictionaryAnalyzer::load(directory)?),
            1024,
        ))
    }

    /// Reuses a loaded dictionary with a separate cache. Capacity zero disables storage.
    pub fn from_shared(dictionary: Arc<DictionaryAnalyzer>, cache_capacity: usize) -> Self {
        Self {
            dictionary,
            cache: Arc::new(ParseCache::new(cache_capacity)),
        }
    }

    /// Borrows the loaded dictionary without copying it.
    pub fn dictionary(&self) -> &DictionaryAnalyzer {
        &self.dictionary
    }

    /// Returns an owning handle to the same immutable dictionary.
    pub fn shared_dictionary(&self) -> Arc<DictionaryAnalyzer> {
        Arc::clone(&self.dictionary)
    }

    /// Returns stable-ranked analyses. Scores are rule weights, not probabilities.
    ///
    /// Parsing runs outside the cache lock. Concurrent misses may compute the same token.
    /// Errors are not cached; returned values can be modified independently of the cache.
    /// Input is trimmed and lowercased; blank input returns no analyses.
    ///
    /// # Errors
    /// Returns a dictionary error if a traversed record or its table reference is invalid.
    pub fn parse(&self, input: &str) -> Result<Vec<Parse>, DictionaryError> {
        let key = normalization::normalize_input(input);
        if key.is_empty() {
            return Ok(Vec::new());
        }
        if let Some(value) = self.cache.get(&key) {
            return Ok(value);
        }
        let mut result =
            normalization::parse_normalized(&key, |word| word::parse(&self.dictionary, word))?;
        ranking::rank(&mut result);
        self.cache.insert(key, result.clone());
        Ok(result)
    }

    /// Returns analyses in rule order without using or updating the cache.
    pub fn parse_unranked(&self, input: &str) -> Result<Vec<Parse>, DictionaryError> {
        normalization::parse(input, |input| word::parse(&self.dictionary, input))
    }

    /// Returns a snapshot of cache size and counters, shared by analyzer clones.
    pub fn cache_info(&self) -> CacheInfo {
        self.cache.info()
    }

    /// Clears cached results and counters for this analyzer and its clones.
    /// An already running parse can repopulate the cache after this call.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}
