use crate::analysis::context::agreement::{all_pos, has_any, nominal_agreement, subject_agreement};
use crate::analysis::parse::Parse;

const INFINITIVE_GOVERNORS: &[&str] = &[
    "мочь",
    "смочь",
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

fn governs_infinitive(governor: &Parse, infinitive: &Parse) -> bool {
    if governor.tag.pos != "VERB" || !has_any(governor, &["indc"]) || infinitive.tag.pos != "INFN" {
        return false;
    }
    INFINITIVE_GOVERNORS.contains(&governor.normal_form.as_str())
        || (governor.normal_form == "быть"
            && has_any(governor, &["futr"])
            && has_any(infinitive, &["impf"]))
}

fn has_subject(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    index: usize,
    governor: &Parse,
) -> bool {
    let subjects = previous_word(tokens, index)
        .map(|i| candidates[i].as_slice())
        .unwrap_or_default();
    all_pos(subjects, &["NOUN", "NPRO"])
        && subjects
            .iter()
            .any(|subject| subject_agreement(subject, governor))
}

fn following_object(candidates: &[Vec<Parse>], target: usize, infinitive: &Parse) -> bool {
    let objects = candidates
        .get(target + 1)
        .map(Vec::as_slice)
        .unwrap_or_default();
    // A genitive alternative can also attach to the competing noun.
    has_any(infinitive, &["tran"])
        && all_pos(objects, &["NOUN", "NPRO"])
        && objects.iter().any(|object| has_any(object, &["accs"]))
        && !objects
            .iter()
            .any(|object| has_any(object, &["gent", "gen2"]))
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
    governors.iter().any(|governor| {
        if !governs_infinitive(governor, parse) {
            return false;
        }
        let explicit_subject = has_subject(tokens, candidates, previous, governor);
        let future = governor.normal_form == "быть" && has_any(governor, &["futr"]);
        explicit_subject
            || (all_pos(governors, &["VERB"])
                && (has_any(governor, &["1per", "2per"])
                    || (future && following_object(candidates, target, parse))))
    })
}

pub fn inverted_subject(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    target: usize,
    noun: &Parse,
) -> bool {
    let Some(previous) = target.checked_sub(1) else {
        return false;
    };
    let governors = &candidates[previous];
    if noun.tag.pos != "NOUN" || !all_pos(governors, &["VERB"]) {
        return false;
    }
    governors.iter().any(|governor| {
        // A competing governed infinitive needs evidence beyond agreement with this noun.
        subject_agreement(noun, governor)
            && !has_subject(tokens, candidates, previous, governor)
            && !candidates[target]
                .iter()
                .any(|parse| governs_infinitive(governor, parse))
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
    let accusative = all_pos(objects, &["NOUN", "NPRO"])
        && objects.iter().any(|object| has_any(object, &["accs"]))
        && objects
            .iter()
            .all(|object| has_any(object, &["nomn", "accs"]));
    // Agreement can instead form a possessive phrase such as "мой дом".
    let modifier_phrase = candidates[target].iter().any(|modifier| {
        ["ADJF", "PRTF"].contains(&modifier.tag.pos.as_str())
            && objects
                .iter()
                .any(|object| nominal_agreement(modifier, object))
    });
    accusative && !modifier_phrase
}
