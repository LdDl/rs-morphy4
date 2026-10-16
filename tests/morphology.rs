use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

use rs_morphy4::{MorphAnalyzer, Parse};
use serde::Deserialize;

static ANALYZER: LazyLock<MorphAnalyzer> = LazyLock::new(|| {
    MorphAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru")).unwrap()
});
static REFERENCE: LazyLock<Reference> =
    LazyLock::new(|| serde_json::from_str(include_str!("fixtures/rules.json")).unwrap());

#[derive(Deserialize)]
struct Reference {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    group: String,
    input: String,
    expected: Vec<Expected>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Expected {
    word: String,
    normal_form: String,
    pos: String,
    grammemes: BTreeSet<String>,
    score: f64,
}

fn compare(group: &str) {
    let mut count = 0;
    for case in REFERENCE.cases.iter().filter(|c| c.group == group) {
        let actual: Vec<_> = ANALYZER
            .parse_unranked(&case.input)
            .unwrap()
            .into_iter()
            .map(|p| Expected {
                word: p.word,
                normal_form: p.normal_form,
                pos: p.tag.pos,
                grammemes: p.tag.grammemes,
                score: p.score,
            })
            .collect();
        assert_eq!(actual, case.expected, "{group}: {:?}", case.input);
        count += 1;
    }
    assert!(count > 0);
}

#[test]
fn special_tokens_match_reference() {
    compare("special");
}
#[test]
fn unknown_words_match_reference() {
    compare("unknown");
}
#[test]
fn compound_rules_match_reference() {
    compare("compound");
}
#[test]
fn known_words_match_full_rule_order() {
    compare("dictionary");
}

fn first(word: &str) -> Parse {
    ANALYZER.parse(word).unwrap().remove(0)
}

#[test]
fn token_results_do_not_invent_dictionary_references() {
    for word in [
        "123",
        "12,5",
        "IV",
        "abc",
        "?!",
        "по-новому",
        "быстро-быстро",
        "наистарейший",
    ] {
        for p in ANALYZER.parse(word).unwrap() {
            assert!(p.paradigm_id.is_none(), "{word}");
            assert!(p.form_index.is_none(), "{word}");
            assert!(p.stem.is_none(), "{word}");
        }
    }
    assert_eq!(first("123").tag.pos, "NUMB");
    assert!(first("12,5").tag.contains("real"));
    assert_eq!(
        ANALYZER
            .parse("IV")
            .unwrap()
            .iter()
            .map(|p| p.tag.pos.as_str())
            .collect::<Vec<_>>(),
        ["ROMN", "LATN"]
    );
}

#[test]
fn dictionary_mode_stays_independent_from_prediction() {
    assert!(ANALYZER.dictionary().parse("жжжщщ").unwrap().is_empty());
    let p = first("ѯѯѯѯ");
    assert_eq!(p.tag.pos, "UNKN");
    assert_eq!(p.score, 0.1);
    assert!(p.paradigm_id.is_none());
    let known = first("кошки");
    assert!(known.paradigm_id.is_some());
    assert!(known.form_index.is_some());
    assert!(known.stem.is_some());
}

#[test]
fn inflection_patterns_can_return_multiple_analyses() {
    let parses = ANALYZER.parse_unranked("ѯой").unwrap();
    assert_eq!(parses.len(), 2);
    assert_eq!(
        (
            parses[0].tag.pos.as_str(),
            parses[0].normal_form.as_str(),
            parses[0].score
        ),
        ("ADJF", "ѯый", 0.6)
    );
    assert_eq!(
        (
            parses[1].tag.pos.as_str(),
            parses[1].normal_form.as_str(),
            parses[1].score
        ),
        ("NOUN", "ѯой", 0.4)
    );
    assert_eq!(first("ѯти").normal_form, "ѯть");
}

#[test]
fn compound_features_and_particles_follow_rule_precedence() {
    let compound = first("команд-участниц");
    assert_eq!(compound.normal_form, "команда-участница");
    assert!(compound.tag.contains("inan"));
    assert!(!compound.tag.contains("anim"));
    let particle = first("скажи-ка");
    assert_eq!(particle.normal_form, "сказать-ка");
    assert_eq!(particle.score, 0.9);
    assert!(particle.paradigm_id.is_some());
    assert_eq!(first("pdf-документов").normal_form, "pdf-документ");
    assert_eq!(first("по-новому").tag.pos, "ADVB");
    assert!(first("наистарейший").tag.contains("Supr"));
}

#[test]
fn predicted_verb_aspect_follows_the_paradigm_without_prefix_overrides() {
    for word in ["фырзуют", "зафырзуют"] {
        let parses = ANALYZER.parse(word).unwrap();
        assert!(parses.iter().any(|p| p.tag.contains("perf")));
        assert!(parses.iter().any(|p| p.tag.contains("impf")));
        for parse in parses {
            let lexeme = ANALYZER.lexeme(&parse).unwrap();
            let source = &lexeme[usize::from(parse.form_index.unwrap())];
            assert_eq!(source.tag, parse.tag);
            assert_eq!(source.word, word);
        }
    }
}

#[test]
fn empty_input_and_long_numeric_tokens_are_supported() {
    assert!(ANALYZER.parse("\t\n ").unwrap().is_empty());
    let number = "9".repeat(10000);
    let parses = ANALYZER.parse(&number).unwrap();
    assert_eq!(parses.len(), 1);
    assert_eq!(parses[0].word, number);
    assert!(parses[0].tag.contains("intg"));
}
