use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

const STEMS: &[&str] = &[
    "к", "м", "п", "т", "л", "н", "р", "с", "в", "д", "б", "г", "з", "ж", "х",
];
const PERFECTIVE_PREFIXES: &[&str] = &[
    "вз", "вс", "вы", "до", "за", "из", "ис", "на", "о", "об", "от", "пере", "по", "под", "при",
    "про", "раз", "рас", "с", "у",
];

pub fn parse(
    dictionary: &DictionaryAnalyzer,
    word: &str,
) -> Result<Option<Parse>, DictionaryError> {
    let positions: Vec<_> = word.char_indices().map(|(i, _)| i).collect();
    if positions.len() < 4 {
        return Ok(None);
    }
    for size in [4, 3, 2] {
        if positions.len() <= size {
            continue;
        }
        let ending = &word[positions[positions.len() - size]..];
        for candidate_stem in STEMS {
            let candidate = format!("{candidate_stem}{ending}");
            if let Some(mut example) = dictionary.parse_exact(&candidate)?.into_iter().next() {
                let (Some(id), Some(index)) = (example.paradigm_id, example.form_index) else {
                    continue;
                };
                let forms = dictionary.paradigms().entries()[usize::from(id)].forms();
                let form = forms[usize::from(index)];
                let tables = dictionary.tables();
                let prefixes = &tables.metadata.compile_options.paradigm_prefixes;
                let suffixes = &tables.suffixes;
                let stem = word
                    .strip_prefix(&prefixes[usize::from(form.prefix_id)])
                    .unwrap_or(word);
                let stem = stem
                    .strip_suffix(&suffixes[usize::from(form.suffix_id)])
                    .unwrap_or(stem);
                let normal = forms[0];
                example.word = word.to_owned();
                example.normal_form = format!(
                    "{}{}{}",
                    prefixes[usize::from(normal.prefix_id)],
                    stem,
                    suffixes[usize::from(normal.suffix_id)]
                );
                example.stem = Some(stem.to_owned());
                example.score = 0.5;
                correct_aspect(&mut example);
                return Ok(Some(example));
            }
        }
    }
    Ok(None)
}

fn correct_aspect(parse: &mut Parse) {
    if matches!(parse.tag.pos.as_str(), "VERB" | "INFN")
        && parse.tag.contains("impf")
        && PERFECTIVE_PREFIXES.iter().any(|p| {
            parse
                .word
                .strip_prefix(p)
                .is_some_and(|s| s.chars().count() > 2)
        })
    {
        parse.tag.grammemes.remove("impf");
        parse.tag.grammemes.insert("perf".into());
    }
}
