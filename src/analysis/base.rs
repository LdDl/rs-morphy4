use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::analysis::patterns;
use crate::analysis::prediction;
use crate::analysis::prefixes;
use crate::analysis::special;
use crate::dictionary::DictionaryError;

pub fn parse(dictionary: &DictionaryAnalyzer, word: &str) -> Result<Vec<Parse>, DictionaryError> {
    let special = special::parse(word);
    if !special.is_empty() {
        return Ok(special);
    }
    let known = dictionary.parse_with_yo(word)?;
    if !known.is_empty() {
        return Ok(known);
    }
    let prefixed = prefixes::parse(dictionary, word)?;
    if !prefixed.is_empty() {
        return Ok(prefixed);
    }
    let predicted = prediction::parse(dictionary, word)?;
    if !predicted.is_empty() {
        return Ok(predicted);
    }
    Ok(patterns::parse(word))
}
