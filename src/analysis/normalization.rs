use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

pub fn parse(
    word: &str,
    analyze: impl Fn(&str) -> Result<Vec<Parse>, DictionaryError>,
) -> Result<Vec<Parse>, DictionaryError> {
    parse_normalized(&normalize_input(word), analyze)
}

pub fn normalize_input(word: &str) -> String {
    word.trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .to_lowercase()
}

pub fn parse_normalized(
    word: &str,
    analyze: impl Fn(&str) -> Result<Vec<Parse>, DictionaryError>,
) -> Result<Vec<Parse>, DictionaryError> {
    if word.is_empty() {
        return Ok(Vec::new());
    }
    analyze(word)
}
