use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

const ENDINGS: &[&str] = &[
    "ейший",
    "ейшая",
    "ейшее",
    "ейшие",
    "ейшего",
    "ейшему",
    "ейшим",
    "ейших",
    "айший",
    "айшая",
    "айшее",
    "айшие",
    "айшего",
    "айшему",
    "айшим",
    "айших",
];

pub fn parse(
    dictionary: &DictionaryAnalyzer,
    word: &str,
) -> Result<Option<Parse>, DictionaryError> {
    let Some(without_prefix) = word.strip_prefix("наи") else {
        return Ok(None);
    };
    for ending in ENDINGS {
        if let Some(stem) = without_prefix.strip_suffix(ending) {
            let negative = stem.strip_prefix("не").filter(|_| stem.chars().count() > 3);
            for variant in negative.into_iter().chain(std::iter::once(stem)) {
                for base_ending in ["ный", "ый", "ий"] {
                    let candidate = format!("{variant}{base_ending}");
                    if let Some(mut base) = dictionary.parse_exact(&candidate)?.into_iter().next() {
                        base.tag.grammemes.insert("Supr".into());
                        return Ok(Some(Parse::synthetic(
                            word,
                            base.normal_form,
                            base.tag,
                            1.0,
                        )));
                    }
                }
            }
        }
    }
    Ok(None)
}
