use std::fs;
use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use rs_morphy4::DictionaryAnalyzer;
use rs_morphy4::dictionary::{BinaryError, DictionaryError, TableKind};

use crate::fixture::DawgFixture;

fn fixture_directory(word: &str, record: [u8; 4]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru");
    for filename in [
        "meta.json",
        "grammemes.json",
        "gramtab-opencorpora-int.json",
        "suffixes.json",
        "paradigms.array",
    ] {
        fs::copy(bundled.join(filename), temp.path().join(filename)).unwrap();
    }
    fs::write(
        temp.path().join("words.dawg"),
        DawgFixture::new(word, STANDARD.encode(record).as_bytes()).bytes(),
    )
    .unwrap();
    temp
}

fn analyzer(word: &str, record: [u8; 4]) -> DictionaryAnalyzer {
    let directory = fixture_directory(word, record);
    DictionaryAnalyzer::load(directory.path()).unwrap()
}

#[test]
fn loaded_analyzer_works_without_prediction_files_or_source_directory() {
    let analyzer = analyzer("ёлка", [0, 8, 0, 0]);
    // The temporary dictionary directory has already been removed.
    let parses = analyzer.parse("ЁЛКА").unwrap();
    assert_eq!(parses.len(), 1);
    assert_eq!(parses[0].normal_form, "ёлка");
    assert_eq!(parses[0].stem.as_deref(), Some("ёл"));
}

#[test]
fn word_records_validate_paradigm_and_form_references() {
    for (record, expected_table) in [
        ([255, 255, 0, 0], TableKind::Paradigm),
        ([0, 8, 255, 255], TableKind::Form),
    ] {
        let error = analyzer("ёлка", record).parse_exact("ёлка").unwrap_err();
        let DictionaryError::BinaryError {
            path,
            source: BinaryError::InvalidReference { table, index, .. },
        } = error
        else {
            panic!("unexpected error: {error:?}")
        };
        assert_eq!(path.file_name().unwrap(), "words.dawg");
        assert_eq!(table, expected_table);
        assert_eq!(index, 65535);
    }
}

#[test]
fn inconsistent_word_affixes_are_not_silently_accepted() {
    let error = analyzer("ёлки", [0, 8, 0, 0])
        .parse_exact("ёлки")
        .unwrap_err();
    assert!(matches!(
        error,
        DictionaryError::BinaryError {
            source: BinaryError::AffixMismatch {
                paradigm: 8,
                form: 0
            },
            ..
        }
    ));
}

#[test]
fn morphology_does_not_turn_dictionary_errors_into_predictions() {
    let directory = fixture_directory("ёлка", [255, 255, 0, 0]);
    let analyzer = rs_morphy4::MorphAnalyzer::load(directory.path()).unwrap();
    let error = analyzer.parse("ёлка").unwrap_err();
    assert!(analyzer.parse("ёлка").is_err());
    assert_eq!(analyzer.cache_info().entries, 0);
    assert_eq!(analyzer.cache_info().misses, 2);
    assert!(matches!(
        error,
        DictionaryError::BinaryError {
            source: BinaryError::InvalidReference {
                table: TableKind::Paradigm,
                ..
            },
            ..
        }
    ));
}
