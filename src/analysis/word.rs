use crate::analysis::base;
use crate::analysis::compound;
use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::analysis::superlative;
use crate::dictionary::DictionaryError;

pub fn parse(dictionary: &DictionaryAnalyzer, word: &str) -> Result<Vec<Parse>, DictionaryError> {
    if word.starts_with("наи")
        && word.chars().count() > 6
        && let Some(parse) = superlative::parse(dictionary, word)?
    {
        return Ok(vec![parse]);
    }
    if let Some(parse) = compound::adverb(word) {
        return Ok(vec![parse]);
    }
    if word.contains('-')
        && let Some(parse) = compound::repeated(dictionary, word)?
    {
        return Ok(vec![parse]);
    }
    let standard = base::parse(dictionary, word)?;
    if standard
        .first()
        .is_some_and(|p| p.tag.pos != "UNKN" && p.score >= 1.0)
    {
        return Ok(standard);
    }
    if word.contains('-') {
        if let Some(parse) = compound::hyphenated(dictionary, word)? {
            return Ok(vec![parse]);
        }
        if let Some(parses) = compound::particle(dictionary, word)? {
            return Ok(parses);
        }
    }
    Ok(standard)
}
