use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

use rs_morphy4::{InflectionError, MorphAnalyzer, Parse};
use serde::Deserialize;

static ANALYZER: LazyLock<MorphAnalyzer> = LazyLock::new(|| {
    MorphAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru"))
        .unwrap()
        .with_probabilities(None)
});
static REFERENCE: LazyLock<Reference> =
    LazyLock::new(|| serde_json::from_str(include_str!("fixtures/inflection.json")).unwrap());

#[derive(Deserialize)]
struct Reference {
    ranked: Vec<Ranked>,
    lexemes: Vec<Lexeme>,
}
#[derive(Deserialize)]
struct Ranked {
    input: String,
    expected: Vec<Expected>,
}
#[derive(Deserialize)]
struct Lexeme {
    input: String,
    index: usize,
    lexeme: Vec<Expected>,
    normalized: Expected,
    inflections: Vec<Inflection>,
    agreements: Vec<Agreement>,
}
#[derive(Deserialize)]
struct Inflection {
    required: Vec<String>,
    result: Option<Expected>,
}
#[derive(Deserialize)]
struct Agreement {
    number: i64,
    result: Option<Expected>,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Expected {
    word: String,
    normal_form: String,
    pos: String,
    grammemes: BTreeSet<String>,
    score: f64,
}

fn pack(p: Parse) -> Expected {
    Expected {
        word: p.word,
        normal_form: p.normal_form,
        pos: p.tag.pos,
        grammemes: p.tag.grammemes,
        score: p.score,
    }
}
fn parse(word: &str) -> Parse {
    ANALYZER.parse(word).unwrap().remove(0)
}

#[test]
fn stable_ranking_matches_reference() {
    for case in &REFERENCE.ranked {
        let actual: Vec<_> = ANALYZER
            .parse(&case.input)
            .unwrap()
            .into_iter()
            .map(pack)
            .collect();
        assert_eq!(actual, case.expected, "{}", case.input);
    }
}

#[test]
fn lexeme_order_duplicates_and_normalized_forms_match_reference() {
    for case in &REFERENCE.lexemes {
        let source = ANALYZER.parse(&case.input).unwrap().remove(case.index);
        let actual: Vec<_> = ANALYZER
            .lexeme(&source)
            .unwrap()
            .into_iter()
            .map(pack)
            .collect();
        assert_eq!(actual, case.lexeme, "{} parse {}", case.input, case.index);
        assert_eq!(
            pack(ANALYZER.normalized(&source).unwrap()),
            case.normalized,
            "{} parse {}",
            case.input,
            case.index
        );
    }
}

#[test]
fn inflection_and_number_agreement_match_reference() {
    for case in &REFERENCE.lexemes {
        let source = ANALYZER.parse(&case.input).unwrap().remove(case.index);
        for query in &case.inflections {
            assert_eq!(
                ANALYZER
                    .inflect(&source, &query.required)
                    .unwrap()
                    .map(pack),
                query.result,
                "{} parse {}, {:?}",
                case.input,
                case.index,
                query.required
            );
        }
        for query in &case.agreements {
            assert_eq!(
                ANALYZER
                    .make_agree_with_number(&source, query.number)
                    .unwrap()
                    .map(pack),
                query.result,
                "{} parse {}, number {}",
                case.input,
                case.index,
                query.number
            );
        }
    }
}

#[test]
fn inflected_forms_keep_paradigm_and_allow_repeated_inflection() {
    let source = parse("кошка");
    let plural = ANALYZER
        .inflect(&source, ["plur", "gent"])
        .unwrap()
        .unwrap();
    assert_eq!(plural.word, "кошек");
    assert_eq!(plural.paradigm_id, source.paradigm_id);
    assert!(plural.form_index.is_some());
    let singular = ANALYZER
        .inflect(&plural, ["sing", "nomn"])
        .unwrap()
        .unwrap();
    assert_eq!(singular.word, "кошка");
    assert_eq!(
        ANALYZER.lexeme(&source).unwrap(),
        ANALYZER.lexeme(&plural).unwrap()
    );
}

#[test]
fn synthetic_and_incompatible_forms_do_not_invent_inflections() {
    let token = parse("hello");
    assert_eq!(
        ANALYZER.lexeme(&token).unwrap().as_slice(),
        std::slice::from_ref(&token)
    );
    assert!(ANALYZER.inflect(&token, ["gent"]).unwrap().is_none());
    assert_eq!(ANALYZER.inflect(&token, ["LATN"]).unwrap(), Some(token));
    assert!(
        ANALYZER
            .inflect(&parse("кошка"), ["sing", "plur"])
            .unwrap()
            .is_none()
    );
}

#[test]
fn malformed_public_parse_metadata_returns_typed_errors() {
    let mut source = parse("кошка");
    source.paradigm_id = Some(u16::MAX);
    assert!(matches!(
        ANALYZER.lexeme(&source),
        Err(InflectionError::InvalidParadigm {
            index: u16::MAX,
            ..
        })
    ));
    source = parse("кошка");
    source.form_index = Some(u16::MAX);
    assert!(matches!(
        ANALYZER.lexeme(&source),
        Err(InflectionError::InvalidForm {
            index: u16::MAX,
            ..
        })
    ));
    source = parse("кошка");
    source.stem = None;
    assert_eq!(
        ANALYZER.lexeme(&source),
        Err(InflectionError::IncompleteReference)
    );
    source = parse("hello");
    source.form_index = Some(0);
    assert_eq!(
        ANALYZER.lexeme(&source),
        Err(InflectionError::IncompleteReference)
    );
}
