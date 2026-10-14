use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::inflection::error::InflectionError;
use crate::analysis::parse::Parse;

pub fn lexeme(
    dictionary: &DictionaryAnalyzer,
    parse: &Parse,
) -> Result<Vec<Parse>, InflectionError> {
    let (id, stem) = match (parse.paradigm_id, parse.stem.as_deref()) {
        (Some(id), Some(stem)) => (id, stem),
        (None, None) if parse.form_index.is_none() => return Ok(vec![parse.clone()]),
        _ => return Err(InflectionError::IncompleteReference),
    };
    let entries = dictionary.paradigms().entries();
    let forms = entries
        .get(usize::from(id))
        .ok_or(InflectionError::InvalidParadigm {
            index: id,
            length: entries.len(),
        })?
        .forms();
    if let Some(index) = parse.form_index
        && usize::from(index) >= forms.len()
    {
        return Err(InflectionError::InvalidForm {
            paradigm: id,
            index,
            length: forms.len(),
        });
    }
    let tables = dictionary.tables();
    let prefixes = &tables.metadata.compile_options.paradigm_prefixes;
    let suffixes = &tables.suffixes;
    let particle = parse
        .normal_form
        .rsplit_once('-')
        .filter(|(_, tail)| ["ка", "то", "таки", "де", "тко", "тка", "с", "ста"].contains(tail))
        .map(|(_, tail)| format!("-{tail}"))
        .unwrap_or_default();
    let first = forms[0];
    let normal_form = format!(
        "{}{}{}{}",
        prefixes[usize::from(first.prefix_id)],
        stem,
        suffixes[usize::from(first.suffix_id)],
        particle
    );
    Ok(forms
        .iter()
        .zip(0u16..)
        .map(|(form, index)| Parse {
            word: format!(
                "{}{}{}{}",
                prefixes[usize::from(form.prefix_id)],
                stem,
                suffixes[usize::from(form.suffix_id)],
                particle
            ),
            normal_form: normal_form.clone(),
            tag: tables.tags[usize::from(form.tag_id)].clone(),
            score: parse.score,
            paradigm_id: Some(id),
            form_index: Some(index),
            stem: Some(stem.to_owned()),
        })
        .collect())
}
