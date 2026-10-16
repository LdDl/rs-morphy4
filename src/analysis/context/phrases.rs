use crate::analysis::context::agreement::{all_pos, has_any, subject_agreement};
use crate::analysis::parse::Parse;

pub fn numeral_agreement(candidates: &[Vec<Parse>], target: usize, noun: &Parse) -> bool {
    let Some(previous) = target.checked_sub(1) else {
        return false;
    };
    let numerals = &candidates[previous];
    if noun.tag.pos != "NOUN" || !all_pos(numerals, &["NUMR"]) {
        return false;
    }
    numerals.iter().any(|numeral| {
        if !["два", "три", "четыре"].contains(&numeral.normal_form.as_str()) {
            return false;
        }
        if has_any(numeral, &["nomn", "accs"]) {
            let gender = numeral.normal_form != "два"
                || !has_any(numeral, &["masc", "femn", "neut"])
                || (has_any(numeral, &["femn"]) == has_any(noun, &["femn"]));
            let animate_accusative = has_any(numeral, &["accs"]) && has_any(numeral, &["anim"]);
            has_any(noun, &["gent"])
                && has_any(
                    noun,
                    if animate_accusative {
                        &["plur"]
                    } else {
                        &["sing"]
                    },
                )
                && gender
        } else {
            has_any(noun, &["plur"])
                && ["gent", "datv", "ablt", "loct"]
                    .iter()
                    .any(|case| has_any(numeral, &[*case]) && has_any(noun, &[*case]))
        }
    })
}

pub fn locative_existence(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    target: usize,
    noun: &Parse,
) -> bool {
    if noun.tag.pos != "NOUN" || !has_any(noun, &["nomn"]) || target < 3 {
        return false;
    }
    let prep = target - 3;
    let at_start = prep == 0 || [".", "!", "?", ";", ":"].contains(&tokens[prep - 1].as_str());
    let at_end =
        target + 1 == tokens.len() || [".", "!", "?", ";"].contains(&tokens[target + 1].as_str());
    if !at_start || !at_end {
        return false;
    }
    let prepositions = &candidates[prep];
    let places = &candidates[prep + 1];
    let governors = &candidates[prep + 2];
    let locative = prepositions
        .iter()
        .any(|p| p.tag.pos == "PREP" && ["в", "на"].contains(&p.normal_form.as_str()))
        && all_pos(places, &["NOUN"])
        && places.iter().any(|place| has_any(place, &["loct", "loc2"]));
    locative
        && all_pos(governors, &["VERB"])
        && governors.iter().any(|verb| {
            verb.normal_form == "быть" && has_any(verb, &["indc"]) && subject_agreement(noun, verb)
        })
}
