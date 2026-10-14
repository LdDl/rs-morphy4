use crate::analysis::parse::Parse;
use crate::analysis::special::decimal::is_decimal;
use crate::analysis::special::latin::is_latin;
use crate::tag::Tag;

pub fn parse(word: &str) -> Vec<Parse> {
    if word.is_empty() {
        return Vec::new();
    }
    if word
        .chars()
        .all(|c| c.is_ascii_punctuation() || matches!(c, '\u{2026}' | '\u{2014}' | '\u{2013}'))
    {
        return vec![token(word, "PNCT", &[])];
    }
    if word.chars().all(is_decimal) {
        return vec![token(word, "NUMB", &["intg"])];
    }
    if let Some((left, right)) = word.split_once(['.', ','])
        && !left.is_empty()
        && !right.is_empty()
        && left.chars().all(is_decimal)
        && right.chars().all(is_decimal)
    {
        return vec![token(word, "NUMB", &["real"])];
    }
    let mut result = Vec::new();
    if is_roman(word) {
        result.push(token(word, "ROMN", &[]));
    }
    if is_latin(word) {
        result.push(token(word, "LATN", &[]));
    }
    result
}

fn is_roman(word: &str) -> bool {
    let upper = word.to_uppercase();
    upper
        .chars()
        .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
        && !["IIII", "VVVV", "XXXX", "LLLL", "CCCC", "DDDD"]
            .iter()
            .any(|s| upper.contains(s))
}

fn token(word: &str, pos: &str, grammemes: &[&str]) -> Parse {
    Parse::synthetic(
        word,
        word.to_owned(),
        Tag::new(pos, grammemes.iter().copied()),
        1.0,
    )
}
