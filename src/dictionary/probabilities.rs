use std::{collections::HashMap, path::Path};

use crate::dictionary::{
    dawg::IntegerDawg,
    error::{DictionaryError, ValidationError},
    probability_tags,
};
use crate::tag::Tag;

const VALUES: &[u8] = include_bytes!("../../data/probabilities_ru/p_t_given_w.intdawg");
const TAGS: &[u8] = include_bytes!("../../data/probabilities_ru/gramtab-opencorpora-int.json");
const VALUE_PATH: &str = "<memory>/p_t_given_w.intdawg";

/// Word/tag frequencies independent of morphological paradigm identifiers.
#[derive(Debug)]
pub struct WordTagProbabilities {
    values: IntegerDawg,
    tags: HashMap<Tag, Vec<String>>,
}

impl WordTagProbabilities {
    /// Loads the embedded Russian frequency model without filesystem access.
    pub fn embedded() -> Result<Self, DictionaryError> {
        Self::from_bytes(VALUES, TAGS)
    }

    /// Decodes an integer completion DAWG and its JSON array of tag spellings.
    /// Keys have the form word:tag; values are probabilities in millionths.
    pub fn from_bytes(values: &[u8], tags: &[u8]) -> Result<Self, DictionaryError> {
        Ok(Self {
            values: IntegerDawg::from_bytes(values, VALUE_PATH)?,
            tags: probability_tags::decode(tags, Path::new("<memory>/probability-tags.json"))?,
        })
    }

    /// Returns a frequency in millionths, or zero for an absent word/tag pair.
    /// Equivalent tag spellings use their maximum value. Input is already normalized.
    pub fn frequency(&self, word: &str, tag: &Tag) -> Result<u32, DictionaryError> {
        let Some(aliases) = self.tags.get(tag) else {
            return Ok(0);
        };
        let mut maximum = 0;
        for alias in aliases {
            let value = self.values.get(&format!("{word}:{alias}"))?.unwrap_or(0);
            if value > 1_000_000 {
                return Err(DictionaryError::invalid(
                    Path::new(VALUE_PATH),
                    ValidationError::InvalidProbability { value },
                ));
            }
            maximum = maximum.max(value);
        }
        Ok(maximum)
    }
}
