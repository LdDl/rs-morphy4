use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

use rs_morphy4::dictionary::dawg::{PredictionRecord, RecordDawg, WordRecord};
use rs_morphy4::{DictionaryAnalyzer, Parse};
use serde::Deserialize;

static ANALYZER: LazyLock<DictionaryAnalyzer> = LazyLock::new(|| {
    DictionaryAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru")).unwrap()
});
static REFERENCE: LazyLock<Reference> =
    LazyLock::new(|| serde_json::from_str(include_str!("fixtures/dictionary.json")).unwrap());

#[derive(Deserialize)]
struct Reference {
    paradigm_count: usize,
    form_count: usize,
    paradigms: Vec<ExpectedParadigm>,
    words: Vec<WordCase>,
    predictions: Vec<PredictionCases>,
}

#[derive(Deserialize)]
struct ExpectedParadigm {
    index: usize,
    forms: Vec<[u16; 3]>,
}

#[derive(Deserialize)]
struct WordCase {
    input: String,
    records: Vec<[u16; 2]>,
    exact: Vec<ExpectedParse>,
    parsed: Vec<ExpectedParse>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ExpectedParse {
    word: String,
    normal_form: String,
    pos: String,
    grammemes: BTreeSet<String>,
    score: f64,
    paradigm_id: u16,
    stem: String,
}

#[derive(Deserialize)]
struct PredictionCases {
    prefix: usize,
    cases: Vec<PredictionCase>,
}

#[derive(Deserialize)]
struct PredictionCase {
    key: String,
    records: Vec<[u32; 3]>,
}

fn comparable(parses: Vec<Parse>) -> Vec<ExpectedParse> {
    parses
        .into_iter()
        .map(|p| ExpectedParse {
            word: p.word,
            normal_form: p.normal_form,
            pos: p.tag.pos,
            grammemes: p.tag.grammemes,
            score: p.score,
            paradigm_id: p.paradigm_id.unwrap(),
            stem: p.stem.unwrap(),
        })
        .collect()
}

#[test]
fn all_paradigms_load_and_selected_forms_match_reference() {
    let entries = ANALYZER.paradigms().entries();
    assert_eq!(entries.len(), REFERENCE.paradigm_count);
    assert_eq!(
        entries.iter().map(|p| p.forms().len()).sum::<usize>(),
        REFERENCE.form_count
    );
    for expected in &REFERENCE.paradigms {
        let forms: Vec<_> = entries[expected.index]
            .forms()
            .iter()
            .map(|f| [f.suffix_id, f.tag_id, f.prefix_id])
            .collect();
        assert_eq!(forms, expected.forms, "paradigm {}", expected.index);
    }
}

#[test]
fn word_records_preserve_reference_order_and_duplicates() {
    let words = RecordDawg::<WordRecord>::load(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru/words.dawg"),
    )
    .unwrap();
    for case in &REFERENCE.words {
        let actual: Vec<_> = words
            .get(&case.input)
            .unwrap()
            .iter()
            .map(|r| [r.paradigm_id, r.form_index])
            .collect();
        assert_eq!(actual, case.records, "{:?}", case.input);
        let exact = ANALYZER.parse_exact(&case.input).unwrap();
        assert_eq!(
            exact
                .iter()
                .map(|p| [p.paradigm_id.unwrap(), p.form_index.unwrap()])
                .collect::<Vec<_>>(),
            actual
        );
    }
}

#[test]
fn exact_and_normalized_analysis_match_dictionary_reference() {
    for case in &REFERENCE.words {
        assert_eq!(
            comparable(ANALYZER.parse_exact(&case.input).unwrap()),
            case.exact,
            "exact {:?}",
            case.input
        );
        assert_eq!(
            comparable(ANALYZER.parse(&case.input).unwrap()),
            case.parsed,
            "normalized {:?}",
            case.input
        );
    }
}

#[test]
fn prediction_files_decode_frequency_and_form_records() {
    for expected in &REFERENCE.predictions {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "data/dicts_ru/prediction-suffixes-{}.dawg",
            expected.prefix
        ));
        let dawg = RecordDawg::<PredictionRecord>::load(path).unwrap();
        for case in &expected.cases {
            let actual: Vec<_> = dawg
                .get(&case.key)
                .unwrap()
                .iter()
                .map(|r| [r.count, u32::from(r.paradigm_id), u32::from(r.form_index)])
                .collect();
            assert_eq!(
                actual, case.records,
                "prefix {}, key {}",
                expected.prefix, case.key
            );
        }
    }
}

#[test]
fn spelling_merge_preserves_input_and_distinct_cases() {
    let parses = ANALYZER.parse("  ОЗЕРА  ").unwrap();
    assert!(
        parses
            .iter()
            .all(|p| p.word == "озера" && p.normal_form == "озеро")
    );
    assert!(
        parses
            .iter()
            .any(|p| p.tag.contains("sing") && p.tag.contains("gent"))
    );
    assert!(
        parses
            .iter()
            .any(|p| p.tag.contains("plur") && p.tag.contains("nomn"))
    );
    let variants = ANALYZER.parse("все").unwrap();
    assert!(variants.iter().any(|p| p.normal_form == "всё"));
    assert!(variants.iter().any(|p| p.normal_form == "весь"));
    assert!(ANALYZER.parse_exact("елка").unwrap().is_empty());
    assert_eq!(ANALYZER.parse("елка").unwrap()[0].normal_form, "ёлка");
    assert!(!ANALYZER.parse("семенов").unwrap().is_empty());
    assert!(!ANALYZER.parse("семёнов").unwrap().is_empty());
}

#[test]
fn unknown_and_special_tokens_have_no_synthetic_analyses() {
    for word in [
        "",
        " \t\n",
        "несуществующеесловоззя",
        "123",
        "hello",
        "!!!",
        "漢字",
        "\0",
        "\u{1}",
        "ёлка\0",
    ] {
        assert!(ANALYZER.parse(word).unwrap().is_empty(), "{word:?}");
    }
}
