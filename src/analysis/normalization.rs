use std::collections::HashSet;

use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

pub fn parse(
    word: &str,
    analyze: impl Fn(&str) -> Result<Vec<Parse>, DictionaryError>,
) -> Result<Vec<Parse>, DictionaryError> {
    let word = word
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
        .to_lowercase();
    if word.is_empty() {
        return Ok(Vec::new());
    }
    let original = analyze(&word)?;
    if !word.contains(['е', 'ё'])
        || original
            .first()
            .is_some_and(|p| p.tag.pos != "UNKN" && p.score >= 1.0 && word.chars().count() > 6)
    {
        return Ok(original);
    }
    let alternate: String = word
        .chars()
        .map(|c| match c {
            'е' => 'ё',
            'ё' => 'е',
            _ => c,
        })
        .collect();
    let mut extra = analyze(&alternate)?;
    if extra.is_empty() {
        return Ok(original);
    }
    let original_keys: HashSet<_> = original.iter().map(|p| (&p.normal_form, &p.tag)).collect();
    let extra_keys: HashSet<_> = extra.iter().map(|p| (&p.normal_form, &p.tag)).collect();
    if original_keys == extra_keys {
        return Ok(original);
    }
    for parse in &mut extra {
        parse.word.clone_from(&word);
    }
    let mut seen = HashSet::new();
    Ok(original
        .into_iter()
        .chain(extra)
        .filter(|p| seen.insert((p.normal_form.clone(), p.tag.clone())))
        .collect())
}
