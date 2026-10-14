use crate::analysis::base;
use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;
use crate::tag::Tag;

pub fn adverb(word: &str) -> Option<Parse> {
    if word.starts_with("по-")
        && word.chars().count() > 4
        && ["ому", "ему", "ски", "цки", "ьи"]
            .iter()
            .any(|s| word.ends_with(s))
    {
        Some(Parse::synthetic(
            word,
            word.to_owned(),
            Tag::new("ADVB", [] as [&str; 0]),
            1.0,
        ))
    } else {
        None
    }
}

pub fn repeated(
    dictionary: &DictionaryAnalyzer,
    word: &str,
) -> Result<Option<Parse>, DictionaryError> {
    if let Some((left, right)) = word.split_once('-')
        && left == right
        && !right.contains('-')
        && let Some(single) = base::parse(dictionary, left)?.into_iter().next()
        && single.tag.pos != "UNKN"
    {
        return Ok(Some(Parse::synthetic(
            word,
            word.to_owned(),
            single.tag,
            1.0,
        )));
    }
    Ok(None)
}

pub fn hyphenated(
    dictionary: &DictionaryAnalyzer,
    word: &str,
) -> Result<Option<Parse>, DictionaryError> {
    let Some((left, right)) = word.split_once('-') else {
        return Ok(None);
    };
    let Some(right_parse) = base::parse(dictionary, right)?.into_iter().next() else {
        return Ok(None);
    };
    if matches!(right_parse.tag.pos.as_str(), "UNKN" | "PRCL") {
        return Ok(None);
    }
    let left_parses = base::parse(dictionary, left)?;
    let mutable = left_parses.first().is_some_and(|p| {
        p.tag.pos != "UNKN"
            && p.score >= 1.0
            && !(matches!(p.tag.pos.as_str(), "ADJS" | "ADVB") && left.ends_with(['о', 'е']))
    });
    let mut tag = right_parse.tag;
    let normal = if mutable {
        let first = &left_parses[0];
        let selected = if matches!(tag.pos.as_str(), "VERB" | "INFN") {
            left_parses
                .iter()
                .find(|p| matches!(p.tag.pos.as_str(), "VERB" | "INFN"))
                .unwrap_or(first)
        } else {
            first
        };
        replace_features(&mut tag, &selected.tag, ["anim", "inan"]);
        replace_features(&mut tag, &selected.tag, ["tran", "intr"]);
        format!("{}-{}", selected.normal_form, right_parse.normal_form)
    } else {
        format!("{left}-{}", right_parse.normal_form)
    };
    Ok(Some(Parse::synthetic(word, normal, tag, 1.0)))
}

pub fn particle(
    dictionary: &DictionaryAnalyzer,
    word: &str,
) -> Result<Option<Vec<Parse>>, DictionaryError> {
    for particle in ["-ка", "-то", "-таки", "-де", "-тко", "-тка", "-с", "-ста"]
    {
        if let Some(base_word) = word.strip_suffix(particle) {
            if base_word.is_empty() {
                return Ok(None);
            }
            let mut parses = base::parse(dictionary, base_word)?;
            if parses.first().is_some_and(|p| p.tag.pos != "UNKN") {
                for parse in &mut parses {
                    parse.word = word.to_owned();
                    parse.normal_form.push_str(particle);
                    parse.score *= 0.9;
                }
                return Ok(Some(parses));
            }
            break;
        }
    }
    Ok(None)
}

fn replace_features(target: &mut Tag, source: &Tag, names: [&str; 2]) {
    if names.iter().any(|g| source.grammemes.contains(*g)) {
        for name in names {
            target.grammemes.remove(name);
            if source.grammemes.contains(name) {
                target.grammemes.insert(name.to_owned());
            }
        }
    }
}
