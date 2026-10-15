use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::analysis::patterns;
use crate::analysis::prediction;
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
    if let Some(predicted) = prediction::parse(dictionary, word)? {
        return Ok(vec![predicted]);
    }
    Ok(patterns::parse(word))
}
