use std::collections::BTreeSet;

use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::inflection::error::InflectionError;
use crate::analysis::inflection::lexeme::lexeme;
use crate::analysis::parse::Parse;
use crate::tag::Tag;

pub fn inflect(
    dictionary: &DictionaryAnalyzer,
    parse: &Parse,
    required: &BTreeSet<String>,
) -> Result<Option<Parse>, InflectionError> {
    let forms = lexeme(dictionary, parse)?;
    let mut candidates: Vec<_> = forms
        .iter()
        .filter(|p| required.iter().all(|g| p.tag.contains(g)))
        .collect();
    if candidates.is_empty() {
        let normalized = Tag::fix_rare_cases(required);
        if normalized != *required {
            candidates = forms
                .iter()
                .filter(|p| normalized.iter().all(|g| p.tag.contains(g)))
                .collect();
        }
    }
    let source = features(&parse.tag);
    let mut best = None;
    let mut best_score = f64::NEG_INFINITY;
    for candidate in candidates {
        let features = features(&candidate.tag);
        let score = source.intersection(&features).count() as f64
            - 0.1 * source.symmetric_difference(&features).count() as f64;
        if score > best_score {
            best = Some(candidate.clone());
            best_score = score;
        }
    }
    Ok(best)
}

fn features(tag: &Tag) -> BTreeSet<&str> {
    tag.grammemes
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(tag.pos.as_str()))
        .collect()
}
