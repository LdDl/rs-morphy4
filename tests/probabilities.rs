use std::sync::Arc;

use rs_morphy4::dictionary::{BinaryError, DictionaryError, ValidationError, dawg::IntegerDawg};
use rs_morphy4::{MorphAnalyzer, Tag, WordTagProbabilities};
use serde::Deserialize;

#[path = "support/integer_dawg.rs"]
mod fixture;
use fixture::IntegerFixture;

#[derive(Deserialize)]
struct Query {
    word: String,
    pos: String,
    grammemes: Vec<String>,
    frequency: u32,
}

#[derive(Deserialize)]
struct Reference {
    queries: Vec<Query>,
    first_lemmas: Vec<(String, Option<String>)>,
}

#[test]
fn integer_dawg_reads_values_and_rejects_corruption() {
    for value in [0, 1, 1_000_000, 0x7fff_ffff] {
        let fixture = IntegerFixture::new("из:PREP", value);
        let dawg = IntegerDawg::from_bytes(&fixture.bytes, "test").unwrap();
        assert_eq!(dawg.get("из:PREP").unwrap(), Some(value));
        for key in ["из", "из:PR", "из:PREP!", "нет", "", "\0"] {
            assert_eq!(dawg.get(key).unwrap(), None);
        }
        for length in [0, 3, 4, fixture.bytes.len() - 1] {
            assert!(IntegerDawg::from_bytes(&fixture.bytes[..length], "test").is_err());
        }
        let mut trailing = fixture.bytes.clone();
        trailing.push(0);
        assert!(IntegerDawg::from_bytes(&trailing, "test").is_err());
    }
    let mut fixture = IntegerFixture::new("из:PREP", 3);
    fixture.set_unit(fixture.value_index, 3);
    let dawg = IntegerDawg::from_bytes(&fixture.bytes, "test").unwrap();
    assert!(matches!(
        dawg.get("из:PREP"),
        Err(DictionaryError::BinaryError {
            source: BinaryError::InvalidLeaf { .. },
            ..
        })
    ));
    fixture.set_unit(
        fixture.terminal,
        (0x1fffff << 10) | (1 << 8) | u32::from(b'P'),
    );
    let dawg = IntegerDawg::from_bytes(&fixture.bytes, "test").unwrap();
    assert!(dawg.get("из:PREP").is_err());
}

#[test]
fn probability_model_matches_fixed_point_reference_and_ranked_corpus() {
    let reference: Reference =
        serde_json::from_str(include_str!("fixtures/probabilities.json")).unwrap();
    let model = WordTagProbabilities::embedded().unwrap();
    assert_eq!(reference.queries.len(), 5914);
    for query in reference.queries {
        let tag = Tag::new(query.pos, query.grammemes);
        assert_eq!(
            model.frequency(&query.word, &tag).unwrap(),
            query.frequency,
            "{} {tag}",
            query.word
        );
    }
    let analyzer = MorphAnalyzer::default();
    assert_eq!(reference.first_lemmas.len(), 2504);
    for (word, expected) in reference.first_lemmas {
        assert_eq!(
            analyzer
                .parse(&word)
                .unwrap()
                .first()
                .map(|p| p.normal_form.clone()),
            expected,
            "{word}"
        );
    }
}

#[test]
fn invalid_model_data_is_reported_without_falling_back() {
    let fixture = IntegerFixture::new("из:PREP", 1_000_001);
    let tags = br#"["PREP"]"#;
    let model = WordTagProbabilities::from_bytes(&fixture.bytes, tags).unwrap();
    assert!(matches!(
        model.frequency("из", &Tag::new("PREP", [] as [&str; 0])),
        Err(DictionaryError::InvalidData {
            reason: ValidationError::InvalidProbability { .. },
            ..
        })
    ));
    let analyzer = MorphAnalyzer::default().with_probabilities(Some(Arc::new(model)));
    assert!(analyzer.parse("из").is_err());
    assert_eq!(analyzer.cache_info().entries, 0);
    for invalid in [b"bad".as_slice(), br#"[""]"#, br#"["NOUN:bad"]"#] {
        assert!(WordTagProbabilities::from_bytes(&fixture.bytes, invalid).is_err());
    }
}

#[test]
fn semantic_tag_matching_preserves_aliases_and_missing_entries() {
    let fixture = IntegerFixture::new("слово:NOUN,sing nomn", 345_678);
    let model =
        WordTagProbabilities::from_bytes(&fixture.bytes, br#"["NOUN,sing nomn","NOUN,nomn sing"]"#)
            .unwrap();
    let tag = Tag::new("NOUN", ["nomn", "sing"]);
    assert_eq!(model.frequency("слово", &tag).unwrap(), 345_678);
    assert_eq!(model.frequency("другое", &tag).unwrap(), 0);
    assert_eq!(
        model
            .frequency("слово", &Tag::new("NOUN", ["plur"]))
            .unwrap(),
        0
    );
}

#[test]
fn replacing_model_isolates_cache_from_existing_clones() {
    let statistical = MorphAnalyzer::default();
    assert_eq!(statistical.parse("из").unwrap()[0].normal_form, "из");
    let original = statistical.clone();
    let heuristic = statistical.with_probabilities(None);
    assert_eq!(heuristic.cache_info().entries, 0);
    assert_eq!(heuristic.parse("из").unwrap()[0].normal_form, "иза");
    assert_eq!(original.parse("из").unwrap()[0].normal_form, "из");
    assert_eq!(original.cache_info().hits, 1);
    let restored =
        heuristic.with_probabilities(Some(Arc::new(WordTagProbabilities::embedded().unwrap())));
    assert_eq!(restored.cache_info().entries, 0);
    assert_eq!(restored.parse("из").unwrap()[0].normal_form, "из");
}

#[test]
fn equal_or_absent_frequencies_preserve_candidates_and_weights() {
    let analyzer = MorphAnalyzer::default();
    let heuristic = analyzer.clone().with_probabilities(None);
    for word in ["нейросервис", "ѯой", "hello", "123", "шушпанчиками"] {
        assert_eq!(
            analyzer.parse(word).unwrap(),
            heuristic.parse(word).unwrap(),
            "{word}"
        );
    }
    for word in ["из", "новыми", "воду", "стали", "три"] {
        let before = heuristic.parse(word).unwrap();
        let after = analyzer.parse(word).unwrap();
        assert_eq!(before.len(), after.len());
        for parse in before {
            assert!(after.contains(&parse));
        }
    }
    let fixture = IntegerFixture::new("из:PREP", 0);
    let model = WordTagProbabilities::from_bytes(&fixture.bytes, br#"["PREP"]"#).unwrap();
    let zero = analyzer.with_probabilities(Some(Arc::new(model)));
    assert_eq!(zero.parse("из").unwrap(), heuristic.parse("из").unwrap());
}
