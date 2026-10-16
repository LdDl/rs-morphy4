use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

const PREFIXES: &[&str] = &[
    "нейро",
    "микро",
    "макро",
    "мега",
    "супер",
    "ультра",
    "анти",
    "псевдо",
];

pub fn parse(dictionary: &DictionaryAnalyzer, word: &str) -> Result<Vec<Parse>, DictionaryError> {
    let mut result = Vec::new();
    for prefix in PREFIXES {
        let Some(remainder) = word.strip_prefix(prefix) else {
            continue;
        };
        if remainder.chars().count() < 3 || !remainder.chars().all(char::is_alphabetic) {
            continue;
        }
        for mut parse in dictionary.parse_with_yo(remainder)? {
            if !["NOUN", "ADJF", "ADJS"].contains(&parse.tag.pos.as_str())
                || ["Abbr", "Name", "Surn", "Patr"]
                    .iter()
                    .any(|feature| parse.tag.contains(feature))
            {
                continue;
            }
            let (Some(paradigm), Some(stem)) = (parse.paradigm_id, parse.stem.as_ref()) else {
                continue;
            };
            let prefixes = &dictionary
                .tables()
                .metadata
                .compile_options
                .paradigm_prefixes;
            // A form prefix would be placed before the added part during inflection.
            if dictionary.paradigms().entries()[usize::from(paradigm)]
                .forms()
                .iter()
                .any(|form| !prefixes[usize::from(form.prefix_id)].is_empty())
            {
                continue;
            }
            parse.word = word.to_owned();
            parse.normal_form = format!("{prefix}{}", parse.normal_form);
            parse.stem = Some(format!("{prefix}{stem}"));
            parse.score = 0.75;
            result.push(parse);
        }
    }
    Ok(result)
}
