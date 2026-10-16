use crate::analysis::context::agreement::{all_pos, has_any, subject_agreement};
use crate::analysis::parse::Parse;

const INFINITIVE_GOVERNORS: &[&str] = &[
    "мочь",
    "хотеть",
    "начать",
    "начинать",
    "продолжать",
    "перестать",
    "пытаться",
];

fn previous_word(tokens: &[String], index: usize) -> Option<usize> {
    let previous = index.checked_sub(1)?;
    // Only one negation particle may intervene; punctuation is never skipped.
    if tokens[previous] == "не" {
        previous.checked_sub(1)
    } else {
        Some(previous)
    }
}

pub fn infinitive_governed(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    target: usize,
    parse: &Parse,
) -> bool {
    if parse.tag.pos != "INFN" {
        return false;
    }
    let Some(previous) = previous_word(tokens, target) else {
        return false;
    };
    let governors = &candidates[previous];
    if !all_pos(governors, &["VERB"]) {
        return false;
    }
    let subjects = previous_word(tokens, previous)
        .map(|index| candidates[index].as_slice())
        .unwrap_or_default();
    governors.iter().any(|governor| {
        if !has_any(governor, &["indc"]) {
            return false;
        }
        let future = governor.normal_form == "быть"
            && has_any(governor, &["futr"])
            && has_any(parse, &["impf"]);
        let modal = INFINITIVE_GOVERNORS.contains(&governor.normal_form.as_str());
        let explicit_subject = all_pos(subjects, &["NOUN", "NPRO"])
            && subjects
                .iter()
                .any(|subject| subject_agreement(subject, governor));
        (future || modal) && (explicit_subject || has_any(governor, &["1per", "2per"]))
    })
}

pub fn imperative_with_object(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    target: usize,
    parse: &Parse,
) -> bool {
    if parse.tag.pos != "VERB" || !has_any(parse, &["impr"]) || !has_any(parse, &["tran"]) {
        return false;
    }
    let start = if target > 0 && tokens[target - 1] == "не" {
        target - 1
    } else {
        target
    };
    if start > 0 && ![".", "!", "?", ";", ":"].contains(&tokens[start - 1].as_str()) {
        return false;
    }
    let objects = candidates
        .get(target + 1)
        .map(Vec::as_slice)
        .unwrap_or_default();
    all_pos(objects, &["NOUN", "NPRO"]) && objects.iter().all(|object| has_any(object, &["accs"]))
}
