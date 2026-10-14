use std::path::Path;
use std::process::Command;

use rs_morphy4::MorphAnalyzer;
use rs_morphy4::dictionary::dawg::{RecordDawg, WordRecord};
use rs_morphy4::dictionary::{BinaryError, DictionaryError};

#[test]
fn embedded_and_external_dictionary_results_match_on_the_corpus() {
    let embedded = MorphAnalyzer::default();
    let custom = MorphAnalyzer::new(rs_morphy4::DictionaryData::bundled()).unwrap();
    let external =
        MorphAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru")).unwrap();
    let corpus: Vec<String> =
        serde_json::from_str(include_str!("../benchmarks/corpus.json")).unwrap();
    for word in corpus {
        let actual = embedded.parse(&word).unwrap();
        let expected = external.parse(&word).unwrap();
        assert_eq!(actual, expected, "{word}");
        assert_eq!(custom.parse(&word).unwrap(), expected, "{word}");
        for (left, right) in actual.iter().zip(&expected) {
            assert_eq!(
                embedded.lexeme(left).unwrap(),
                external.lexeme(right).unwrap(),
                "{word}"
            );
        }
    }
    assert_eq!(embedded.cache_info().capacity, 1024);
}

#[test]
fn embedded_analyzer_works_from_an_empty_directory() {
    if std::env::var_os("RS_MORPHY4_EMBEDDED_CHILD").is_some() {
        assert_eq!(std::fs::read_dir(".").unwrap().count(), 0);
        let morph = MorphAnalyzer::default();
        let parse = morph.parse("кошки").unwrap().remove(0);
        assert_eq!(parse.normal_form, "кошка");
        assert_eq!(
            morph
                .inflect(&parse, ["plur", "gent"])
                .unwrap()
                .unwrap()
                .word,
            "кошек"
        );
        assert_eq!(std::fs::read_dir(".").unwrap().count(), 0);
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "embedded_analyzer_works_from_an_empty_directory",
            "--nocapture",
        ])
        .env("RS_MORPHY4_EMBEDDED_CHILD", "1")
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn in_memory_dawg_errors_keep_the_source_label() {
    let error = RecordDawg::<WordRecord>::from_bytes(&[], "memory/words.dawg").unwrap_err();
    assert!(
        matches!(error, DictionaryError::BinaryError { path, source: BinaryError::UnexpectedEnd { .. } } if path == Path::new("memory/words.dawg"))
    );
}

#[test]
fn custom_dictionary_data_is_used_and_input_buffers_can_be_dropped() {
    let bundled = MorphAnalyzer::default();
    let original = bundled.parse("кошки").unwrap().remove(0);
    let paradigm =
        &bundled.dictionary().paradigms().entries()[usize::from(original.paradigm_id.unwrap())];
    let normal_suffix_id = usize::from(paradigm.forms()[0].suffix_id);
    assert_ne!(
        normal_suffix_id,
        usize::from(paradigm.forms()[usize::from(original.form_index.unwrap())].suffix_id)
    );
    let analyzer = {
        let mut suffixes = bundled.dictionary().tables().suffixes.clone();
        suffixes[normal_suffix_id] = "CUSTOM".to_owned();
        let bytes = serde_json::to_vec(&suffixes).unwrap();
        let data = rs_morphy4::DictionaryData {
            suffixes: &bytes,
            ..rs_morphy4::DictionaryData::bundled()
        };
        MorphAnalyzer::new(data).unwrap()
    };
    let custom = analyzer.parse("кошки").unwrap().remove(0);
    assert_eq!(custom.word, original.word);
    assert_eq!(custom.normal_form, "кошCUSTOM");
    assert_eq!(original.normal_form, "кошка");
}

#[test]
fn malformed_custom_dictionary_files_return_errors_without_falling_back() {
    use rs_morphy4::DictionaryData;
    let cases = [
        DictionaryData {
            metadata: b"invalid",
            ..DictionaryData::bundled()
        },
        DictionaryData {
            grammemes: b"invalid",
            ..DictionaryData::bundled()
        },
        DictionaryData {
            tags: b"invalid",
            ..DictionaryData::bundled()
        },
        DictionaryData {
            suffixes: b"invalid",
            ..DictionaryData::bundled()
        },
        DictionaryData {
            paradigms: &[],
            ..DictionaryData::bundled()
        },
        DictionaryData {
            words: &[],
            ..DictionaryData::bundled()
        },
    ];
    for (index, data) in cases.into_iter().enumerate() {
        let error = MorphAnalyzer::new(data).unwrap_err();
        if index < 4 {
            assert!(
                matches!(error, DictionaryError::ParseError { .. }),
                "{error}"
            );
        } else {
            assert!(
                matches!(error, DictionaryError::BinaryError { .. }),
                "{error}"
            );
        }
    }
}

#[test]
fn custom_metadata_selects_the_supplied_tag_table_and_is_validated() {
    use rs_morphy4::DictionaryData;
    use rs_morphy4::dictionary::ValidationError;
    let data = DictionaryData::bundled();
    let original = std::str::from_utf8(data.metadata).unwrap();
    let renamed = original.replace("gramtab-opencorpora-int.json", "custom-tags.json");
    let custom = MorphAnalyzer::new(DictionaryData {
        metadata: renamed.as_bytes(),
        ..data
    })
    .unwrap();
    assert_eq!(
        custom.parse("кошки").unwrap(),
        MorphAnalyzer::default().parse("кошки").unwrap()
    );
    let invalid = original.replace("gramtab-opencorpora-int.json", "../custom-tags.json");
    let error = MorphAnalyzer::new(DictionaryData {
        metadata: invalid.as_bytes(),
        ..data
    })
    .unwrap_err();
    assert!(matches!(
        error,
        DictionaryError::InvalidData {
            reason: ValidationError::InvalidTableFilename { .. },
            ..
        }
    ));
}
