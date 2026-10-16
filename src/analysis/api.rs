use std::collections::{BTreeSet, HashSet};

use crate::analysis::analyzer::MorphAnalyzer;
use crate::analysis::inflection::{self, InflectionError};
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;
use crate::tag::Tag;

impl MorphAnalyzer {
    /// Returns tags in ranked analysis order, including duplicates.
    pub fn tag(&self, word: &str) -> Result<Vec<Tag>, DictionaryError> {
        Ok(self.parse(word)?.into_iter().map(|p| p.tag).collect())
    }

    /// Returns distinct normal forms in ranked analysis order.
    pub fn normal_forms(&self, word: &str) -> Result<Vec<String>, DictionaryError> {
        let mut seen = HashSet::new();
        Ok(self
            .parse(word)?
            .into_iter()
            .map(|p| p.normal_form)
            .filter(|s| seen.insert(s.clone()))
            .collect())
    }

    /// Tests dictionary membership after case, whitespace and е/ё normalization.
    pub fn word_is_known(&self, word: &str) -> Result<bool, DictionaryError> {
        Ok(!self.dictionary().parse(word)?.is_empty())
    }

    /// Returns the first form of the same paradigm, preserving its score and stem.
    /// Synthetic analyses retain their tag and use their stored normal form as the word.
    /// Invalid or incomplete references return an inflection error.
    pub fn normalized(&self, parse: &Parse) -> Result<Parse, InflectionError> {
        let mut result = self.lexeme(parse)?.remove(0);
        result.word.clone_from(&result.normal_form);
        Ok(result)
    }

    /// Expands a parse from this dictionary in paradigm order, preserving duplicates.
    /// Synthetic analyses without a paradigm return only the supplied form.
    ///
    /// # Errors
    /// Returns an error if the parse contains an incomplete or out-of-range dictionary reference.
    pub fn lexeme(&self, parse: &Parse) -> Result<Vec<Parse>, InflectionError> {
        inflection::lexeme(self.dictionary(), parse)
    }

    /// Selects a form satisfying all features, then maximizes similarity to the source.
    /// Rare cases are mapped to common cases only if no exact candidate exists.
    /// No matching form produces `None`; a malformed parse reference produces an error.
    /// The parse must belong to this dictionary; its origin is not checked.
    pub fn inflect<I, S>(
        &self,
        parse: &Parse,
        required: I,
    ) -> Result<Option<Parse>, InflectionError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let required: BTreeSet<String> = required
            .into_iter()
            .map(|g| g.as_ref().to_owned())
            .collect();
        inflection::inflect(self.dictionary(), parse, &required)
    }

    /// Selects sing/nomn, sing/gent or plur/gent using the last decimal digits.
    /// Negative counts use Euclidean remainders, without taking an absolute value.
    pub fn make_agree_with_number(
        &self,
        parse: &Parse,
        number: i64,
    ) -> Result<Option<Parse>, InflectionError> {
        self.inflect(parse, inflection::grammemes(number))
    }
}
