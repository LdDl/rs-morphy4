use crate::analysis::parse::Parse;
use crate::tag::Tag;

const VERBS: &[(&str, &str, &[&str])] = &[
    ("ть", "INFN", &[]),
    ("ти", "INFN", &[]),
    ("чь", "INFN", &[]),
    ("ет", "VERB", &["3per", "sing", "pres"]),
    ("ут", "VERB", &["3per", "plur", "pres"]),
    ("ют", "VERB", &["3per", "plur", "pres"]),
];
const ADJECTIVES: &[(&str, &str)] = &[
    ("ый", "masc"),
    ("ий", "masc"),
    ("ой", "masc"),
    ("ая", "femn"),
    ("яя", "femn"),
    ("ое", "neut"),
    ("ее", "neut"),
];
const NOUNS: &[(&str, &[&str])] = &[
    ("а", &["femn", "nomn", "sing", "inan"]),
    ("ы", &["femn", "nomn", "plur", "inan"]),
    ("ой", &["femn", "gent", "sing", "inan"]),
    ("ем", &["masc", "ablt", "sing", "inan"]),
    ("ами", &["femn", "ablt", "plur", "inan"]),
];

pub fn parse(word: &str) -> Vec<Parse> {
    let mut result = Vec::new();
    for &(ending, pos, grammemes) in VERBS {
        if let Some(stem) = word.strip_suffix(ending).filter(|s| !s.is_empty()) {
            let normal = if pos == "INFN" {
                format!("{stem}ть")
            } else {
                word.to_owned()
            };
            result.push(Parse::synthetic(
                word,
                normal,
                Tag::new(pos, grammemes.iter().copied()),
                0.6,
            ));
        }
    }
    for &(ending, gender) in ADJECTIVES {
        if let Some(stem) = word.strip_suffix(ending).filter(|s| !s.is_empty()) {
            result.push(Parse::synthetic(
                word,
                format!("{stem}ый"),
                Tag::new("ADJF", [gender, "nomn", "sing"]),
                0.6,
            ));
        }
    }
    for &(ending, grammemes) in NOUNS {
        if word.strip_suffix(ending).is_some_and(|s| !s.is_empty()) {
            result.push(Parse::synthetic(
                word,
                word.to_owned(),
                Tag::new("NOUN", grammemes.iter().copied()),
                0.4,
            ));
        }
    }
    if result.is_empty() {
        result.push(Parse::synthetic(word, word.to_owned(), Tag::default(), 0.1));
    }
    result
}
