use crate::analysis::parse::Parse;

const CASE: &[&str] = &[
    "nomn", "gent", "datv", "accs", "ablt", "loct", "gen2", "loc2", "acc2", "voct",
];
const NUMBER: &[&str] = &["sing", "plur"];
const GENDER: &[&str] = &["masc", "femn", "neut"];
const PERSON: &[&str] = &["1per", "2per", "3per"];

pub fn has_any(parse: &Parse, features: &[&str]) -> bool {
    features
        .iter()
        .any(|feature| parse.tag.grammemes.contains(*feature))
}

pub fn all_pos(parses: &[Parse], allowed: &[&str]) -> bool {
    !parses.is_empty()
        && parses
            .iter()
            .all(|parse| allowed.contains(&parse.tag.pos.as_str()))
}

fn compatible(left: &Parse, right: &Parse, group: &[&str]) -> bool {
    !has_any(left, group)
        || !has_any(right, group)
        || group.iter().any(|feature| {
            left.tag.grammemes.contains(*feature) && right.tag.grammemes.contains(*feature)
        })
}

pub fn nominal_agreement(modifier: &Parse, noun: &Parse) -> bool {
    compatible(modifier, noun, CASE)
        && compatible(modifier, noun, NUMBER)
        && (has_any(modifier, &["plur"])
            || has_any(noun, &["plur"])
            || compatible(modifier, noun, GENDER))
}

pub fn subject_agreement(subject: &Parse, verb: &Parse) -> bool {
    if !has_any(subject, &["nomn"]) || has_any(verb, &["impr"]) {
        return false;
    }
    if subject.tag.pos == "NOUN" && has_any(verb, &["1per", "2per"]) {
        return false;
    }
    let group = if has_any(verb, &["past"]) {
        GENDER
    } else {
        PERSON
    };
    compatible(subject, verb, NUMBER) && compatible(subject, verb, group)
}

pub fn preposition_cases(word: &str) -> &'static [&'static str] {
    match word {
        "из" => &["gent", "gen2"],
        "без" | "для" | "до" | "от" | "у" | "около" | "возле" | "после" => {
            &["gent"]
        }
        "к" => &["datv"],
        "над" | "перед" => &["ablt"],
        "в" | "на" => &["accs", "loct", "loc2"],
        _ => &[],
    }
}
