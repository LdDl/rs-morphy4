use std::error::Error;
use std::fs;
use std::path::Path;

use rs_morphy4::dictionary::{DictionaryError, DictionaryTables, ValidationError};
use serde_json::{Value, json};
use tempfile::TempDir;

fn fixture() -> TempDir {
    let directory = tempfile::tempdir().unwrap();
    let metadata: Value = serde_json::from_str(include_str!("../data/dicts_ru/meta.json")).unwrap();
    write_json(directory.path(), "meta.json", metadata);
    set_metadata(directory.path(), "gramtab_length", json!(2));
    set_metadata(directory.path(), "suffixes_length", json!(3));
    write_json(
        directory.path(),
        "grammemes.json",
        json!([
            ["POST", "", "ЧР", "часть речи"],
            ["NOUN", "POST", "СУЩ", "существительное"],
            ["sing", "", "ед", "единственное число"],
            ["nomn", "", "им", "именительный падеж"]
        ]),
    );
    write_json(
        directory.path(),
        "gramtab-opencorpora-int.json",
        json!(["NOUN sing,nomn", "NOUN sing"]),
    );
    write_json(directory.path(), "suffixes.json", json!(["", "а", "ё"]));
    directory
}

fn write_json(directory: &Path, name: &str, value: Value) {
    fs::write(directory.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
}

fn set_metadata(directory: &Path, key: &str, value: Value) {
    let mut pairs: Vec<(String, Value)> =
        serde_json::from_slice(&fs::read(directory.join("meta.json")).unwrap()).unwrap();
    let entry = pairs.iter_mut().find(|(name, _)| name == key).unwrap();
    entry.1 = value;
    write_json(directory, "meta.json", json!(pairs));
}

fn assert_invalid(directory: &Path, filename: &str, expected: ValidationError) {
    let error = DictionaryTables::load(directory).unwrap_err();
    let DictionaryError::InvalidData { path, reason } = error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(path, directory.join(filename));
    assert_eq!(reason, expected);
}

#[test]
fn bundled_tables_load_with_original_indices() {
    let tables =
        DictionaryTables::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru"))
            .unwrap();
    assert_eq!(tables.metadata.source_revision, "417260");
    assert_eq!(tables.metadata.paradigms_length, 3472);
    assert_eq!(tables.metadata.words_dawg_length, 5_140_921);
    assert_eq!(
        tables.metadata.compile_options.paradigm_prefixes,
        ["", "по", "наи"]
    );
    assert_eq!(tables.grammemes.len(), 115);
    assert_eq!(tables.grammemes[1].name, "NOUN");
    assert_eq!(tables.grammemes[1].parent, "POST");
    assert_eq!(tables.tags.len(), 5560);
    assert_eq!(tables.tags[0].pos, "NOUN");
    assert!(tables.tags[0].contains("anim"));
    assert_eq!(tables.tags[0].case(), Some("nomn"));
    assert_eq!(tables.tags[1].case(), Some("gent"));
    assert_eq!(tables.suffixes.len(), 16448);
    assert_eq!(&tables.suffixes[..3], ["", "-2", "-бур"]);
}

#[test]
fn json_tables_load_without_binary_dictionary_files() {
    let directory = fixture();
    let tables = DictionaryTables::load(directory.path()).unwrap();
    assert_eq!(tables.tags[0].number(), Some("sing"));
    assert_eq!(tables.tags[1].case(), None);
    assert_eq!(tables.grammemes[1].alias, "СУЩ");
    assert_eq!(tables.suffixes, ["", "а", "ё"]);
}

#[test]
fn metadata_table_filename_is_used() {
    let directory = fixture();
    fs::rename(
        directory.path().join("gramtab-opencorpora-int.json"),
        directory.path().join("tags.json"),
    )
    .unwrap();
    set_metadata(
        directory.path(),
        "gramtab_formats",
        json!({"opencorpora-int": "tags.json"}),
    );
    assert_eq!(
        DictionaryTables::load(directory.path()).unwrap().tags.len(),
        2
    );
}

#[test]
fn metadata_rejects_unsupported_or_inconsistent_settings() {
    for (key, value, expected) in [
        (
            "format_version",
            json!("2.5"),
            ValidationError::UnsupportedFormat {
                found: "2.5".into(),
                expected: "2.4".into(),
            },
        ),
        (
            "language_code",
            json!("uk"),
            ValidationError::UnsupportedLanguage {
                found: "uk".into(),
                expected: "ru".into(),
            },
        ),
        (
            "gramtab_formats",
            json!({}),
            ValidationError::MissingTable {
                format: "opencorpora-int".into(),
            },
        ),
        (
            "gramtab_formats",
            json!({"opencorpora-int": "../tags.json"}),
            ValidationError::InvalidTableFilename {
                filename: "../tags.json".into(),
            },
        ),
        (
            "gramtab_formats",
            json!({"opencorpora-int": "/tmp/tags.json"}),
            ValidationError::InvalidTableFilename {
                filename: "/tmp/tags.json".into(),
            },
        ),
        (
            "prediction_suffixes_dawg_lengths",
            json!([10]),
            ValidationError::LengthMismatch {
                expected: 3,
                actual: 1,
            },
        ),
    ] {
        let directory = fixture();
        set_metadata(directory.path(), key, value);
        assert_invalid(directory.path(), "meta.json", expected);
    }
}

#[test]
fn duplicate_metadata_keys_are_rejected() {
    let directory = fixture();
    let path = directory.path().join("meta.json");
    let mut metadata: Vec<Value> = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    metadata.push(json!(["source_revision", "other"]));
    write_json(directory.path(), "meta.json", json!(metadata));
    assert_invalid(
        directory.path(),
        "meta.json",
        ValidationError::DuplicateMetadataKey {
            key: "source_revision".into(),
        },
    );
}

#[test]
fn table_lengths_must_match_metadata() {
    for (key, filename, actual) in [
        ("gramtab_length", "gramtab-opencorpora-int.json", 2),
        ("suffixes_length", "suffixes.json", 3),
    ] {
        let directory = fixture();
        set_metadata(directory.path(), key, json!(99));
        assert_invalid(
            directory.path(),
            filename,
            ValidationError::LengthMismatch {
                expected: 99,
                actual,
            },
        );
    }
}

#[test]
fn grammeme_names_and_parent_references_are_validated() {
    for (definitions, expected) in [
        (
            json!([["", "", "", ""]]),
            ValidationError::EmptyGrammemeName,
        ),
        (
            json!([["NOUN", "", "", ""], ["NOUN", "", "", ""]]),
            ValidationError::DuplicateGrammemeName {
                name: "NOUN".into(),
            },
        ),
        (
            json!([["NOUN", "missing", "", ""]]),
            ValidationError::UnknownParent {
                name: "NOUN".into(),
                parent: "missing".into(),
            },
        ),
    ] {
        let directory = fixture();
        write_json(directory.path(), "grammemes.json", definitions);
        assert_invalid(directory.path(), "grammemes.json", expected);
    }
}

#[test]
fn tags_reject_unknown_pos_features_and_empty_components() {
    for (tag, unknown) in [
        ("missing sing", "missing"),
        ("NOUN missing", "missing"),
        ("NOUN,,sing", ""),
        ("", ""),
        ("NOUN sing,", ""),
    ] {
        let directory = fixture();
        write_json(
            directory.path(),
            "gramtab-opencorpora-int.json",
            json!([tag, "NOUN"]),
        );
        assert_invalid(
            directory.path(),
            "gramtab-opencorpora-int.json",
            ValidationError::UnknownGrammeme {
                name: unknown.into(),
                tag_index: 0,
            },
        );
    }
}

#[test]
fn file_errors_include_the_path_and_cause() {
    for filename in [
        "meta.json",
        "grammemes.json",
        "gramtab-opencorpora-int.json",
        "suffixes.json",
    ] {
        let directory = fixture();
        fs::remove_file(directory.path().join(filename)).unwrap();
        let error = DictionaryTables::load(directory.path()).unwrap_err();
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .is_some()
        );
        let DictionaryError::IoError { path, source } = error else {
            panic!("unexpected error: {error:?}");
        };
        assert_eq!(path, directory.path().join(filename));
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    }
}

#[test]
fn invalid_json_and_record_shapes_return_decode_errors() {
    for (filename, data) in [
        ("meta.json", "{}"),
        ("grammemes.json", "[[\"NOUN\", \"\", \"alias\"]]"),
        ("gramtab-opencorpora-int.json", "[42]"),
        ("suffixes.json", "["),
    ] {
        let directory = fixture();
        fs::write(directory.path().join(filename), data).unwrap();
        let error = DictionaryTables::load(directory.path()).unwrap_err();
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<serde_json::Error>()
                .is_some()
        );
        let DictionaryError::ParseError { path, source } = error else {
            panic!("unexpected error: {error:?}");
        };
        assert_eq!(path, directory.path().join(filename));
        assert_eq!(source.line(), 1);
    }
}
