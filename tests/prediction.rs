use std::sync::LazyLock;

use rs_morphy4::dictionary::{DictionaryError, ValidationError};
use rs_morphy4::{DictionaryData, InflectionError, MorphAnalyzer};

static MORPH: LazyLock<MorphAnalyzer> = LazyLock::new(MorphAnalyzer::default);

#[test]
fn unfamiliar_inflected_words_retain_the_expected_lemma_and_part_of_speech() {
    let cases = [
        ("шушпанчиками", "шушпанчик", "NOUN"),
        ("шушпанчиков", "шушпанчик", "NOUN"),
        ("шушпанчику", "шушпанчик", "NOUN"),
        ("задеплоилось", "задеплоиться", "VERB"),
        ("задеплоились", "задеплоиться", "VERB"),
        ("задеплоится", "задеплоиться", "VERB"),
        ("кракозябрами", "кракозябра", "NOUN"),
        ("бубляшками", "бубляшка", "NOUN"),
        ("бубляшке", "бубляшка", "NOUN"),
        ("квакозябрили", "квакозябрить", "VERB"),
        ("квакозябрить", "квакозябрить", "INFN"),
        ("бубляшечный", "бубляшечный", "ADJF"),
        ("бубляшечного", "бубляшечный", "ADJF"),
        ("бубляшечными", "бубляшечный", "ADJF"),
    ];
    for (word, lemma, pos) in cases {
        assert!(!MORPH.word_is_known(word).unwrap(), "{word}");
        let parses = MORPH.parse(word).unwrap();
        assert_eq!(parses[0].normal_form, lemma, "{word}");
        assert_eq!(parses[0].tag.pos, pos, "{word}");
        for parse in &parses {
            assert!(parse.score > 0.0 && parse.score < 0.5, "{word}");
            assert!(
                MORPH.lexeme(parse).unwrap().iter().any(|form| {
                    form.word.replace('ё', "е") == word.replace('ё', "е") && form.tag == parse.tag
                }),
                "{word}: {parse:?}"
            );
        }
    }
}

#[test]
fn productive_predictions_keep_alternatives_and_frequency_order() {
    let parses = MORPH.parse("шушпанчиков").unwrap();
    assert!(parses.len() > 1);
    assert!(parses.windows(2).all(|pair| pair[0].score >= pair[1].score));
    for word in ["шушпанчиками", "бубляшками", "фырзой", "задеплоилось"]
    {
        for parse in MORPH.parse(word).unwrap() {
            for forbidden in [
                "NUMR", "NPRO", "PRED", "PREP", "CONJ", "PRCL", "INTJ", "Apro",
            ] {
                assert!(!parse.tag.contains(forbidden), "{word}: {parse:?}");
            }
        }
    }
}

#[test]
fn predicted_forms_can_be_inflected_without_losing_the_stem() {
    for (word, required, expected) in [
        ("шушпанчиками", ["sing", "datv"], "шушпанчику"),
        ("бубляшками", ["plur", "gent"], "бубляшек"),
        ("задеплоилось", ["past", "plur"], "задеплоились"),
        ("бубляшечного", ["plur", "ablt"], "бубляшечными"),
    ] {
        let parse = MORPH.parse(word).unwrap().remove(0);
        let inflected = MORPH.inflect(&parse, required).unwrap().unwrap();
        assert_eq!(inflected.word, expected);
        assert_eq!(inflected.paradigm_id, parse.paradigm_id);
        assert_eq!(inflected.stem, parse.stem);
        assert_eq!(inflected.score, parse.score);
    }
}

#[test]
fn normalization_keeps_the_selected_paradigm_instead_of_reanalyzing_the_lemma() {
    for word in [
        "стали",
        "пила",
        "фырзуют",
        "зафырзуют",
        "задеплоилось",
        "нейросервисами",
        "скажи-ка",
    ] {
        for parse in MORPH.parse(word).unwrap() {
            let normalized = MORPH.normalized(&parse).unwrap();
            assert_eq!(normalized, MORPH.lexeme(&parse).unwrap()[0], "{word}");
            assert_eq!(normalized.paradigm_id, parse.paradigm_id);
            assert_eq!(normalized.score, parse.score);
            assert_eq!(MORPH.normalized(&normalized).unwrap(), normalized);
        }
    }
    let synthetic = MORPH.parse("по-новому").unwrap().remove(0);
    assert_eq!(
        MORPH.normalized(&synthetic).unwrap().word,
        synthetic.normal_form
    );
    let mut invalid = MORPH.parse("кошки").unwrap().remove(0);
    invalid.stem = None;
    assert_eq!(
        MORPH.normalized(&invalid),
        Err(InflectionError::IncompleteReference)
    );
}

#[test]
fn suffix_prediction_is_optional_for_custom_data() {
    let without = MorphAnalyzer::new(DictionaryData {
        prediction_suffixes: None,
        ..DictionaryData::bundled()
    })
    .unwrap();
    assert!(without.dictionary().prediction_suffixes().is_empty());
    assert!(
        without
            .parse("шушпанчиками")
            .unwrap()
            .iter()
            .all(|p| p.paradigm_id.is_none())
    );
    assert_eq!(
        without.parse("кошки").unwrap(),
        MORPH.parse("кошки").unwrap()
    );
    assert_eq!(without.parse("ѯѯѯѯ").unwrap()[0].tag.pos, "UNKN");
}

#[test]
fn malformed_or_incomplete_prediction_data_is_reported() {
    let empty: &[&[u8]] = &[];
    let error = MorphAnalyzer::new(DictionaryData {
        prediction_suffixes: Some(empty),
        ..DictionaryData::bundled()
    })
    .unwrap_err();
    assert!(matches!(
        error,
        DictionaryError::InvalidData {
            reason: ValidationError::LengthMismatch {
                expected: 3,
                actual: 0
            },
            ..
        }
    ));
    let corrupt: &[&[u8]] = &[b"invalid", b"invalid", b"invalid"];
    let error = MorphAnalyzer::new(DictionaryData {
        prediction_suffixes: Some(corrupt),
        ..DictionaryData::bundled()
    })
    .unwrap_err();
    assert!(matches!(error, DictionaryError::BinaryError { path, .. }
        if path.ends_with("prediction-suffixes-0.dawg")));
}
