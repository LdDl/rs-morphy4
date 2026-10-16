use std::collections::HashSet;

use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::dawg::WordRecord;
use crate::dictionary::{BinaryError, DictionaryError};

const NON_PRODUCTIVE: &[&str] = &[
    "NUMR", "NPRO", "PRED", "PREP", "CONJ", "PRCL", "INTJ", "Apro",
];

pub fn parse(dictionary: &DictionaryAnalyzer, word: &str) -> Result<Vec<Parse>, DictionaryError> {
    let positions: Vec<_> = word.char_indices().map(|(index, _)| index).collect();
    if positions.len() < 4 {
        return Ok(Vec::new());
    }
    let options = &dictionary.tables().metadata.compile_options;
    let max_length = options.max_suffix_length.min(positions.len());
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for (prefix_id, model) in dictionary.prediction_suffixes().iter().enumerate().rev() {
        if !word.starts_with(&options.paradigm_prefixes[prefix_id]) {
            continue;
        }
        for length in (1..=max_length).rev() {
            let split = positions[positions.len() - length];
            let mut candidates = Vec::new();
            let mut total = 1.0;
            for matched in model.records.get_with_yo(&word[split..])? {
                let fixed = format!("{}{}", &word[..split], matched.word);
                for record in matched.records {
                    let mut parse = match dictionary.resolve(
                        &fixed,
                        WordRecord {
                            paradigm_id: record.paradigm_id,
                            form_index: record.form_index,
                        },
                    ) {
                        Ok(parse) => parse,
                        // A suffix may match while the complete paradigm affixes do not.
                        Err(BinaryError::AffixMismatch { .. }) => continue,
                        Err(error) => return Err(DictionaryError::binary(&model.path, error)),
                    };
                    if record.count == 0 || NON_PRODUCTIVE.iter().any(|g| parse.tag.contains(g)) {
                        continue;
                    }
                    total += f64::from(record.count);
                    if !seen.insert((fixed.clone(), parse.tag.clone(), record.paradigm_id)) {
                        continue;
                    }
                    parse.word = word.to_owned();
                    parse.score = f64::from(record.count);
                    candidates.push(parse);
                }
            }
            if total > 1.0 {
                for mut parse in candidates {
                    parse.score = 0.5 * parse.score / total;
                    result.push(parse);
                }
                break;
            }
        }
    }
    result.sort_by(|left, right| right.score.total_cmp(&left.score));
    Ok(result)
}
