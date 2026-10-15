#[path = "dawg/yo_spelling.rs"]
mod yo_spelling;

#[path = "dawg/analysis_errors.rs"]
mod analysis_errors;

use std::error::Error;
use std::fs;

use rs_morphy4::dictionary::dawg::{PredictionRecord, RecordDawg, WordRecord};
use rs_morphy4::dictionary::{BinaryError, DictionaryError};

#[path = "support/dawg.rs"]
mod fixture;
use fixture::DawgFixture;

fn lookup(data: &[u8], key: &str) -> Result<Vec<WordRecord>, DictionaryError> {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("words.dawg");
    fs::write(&path, data).unwrap();
    RecordDawg::<WordRecord>::load(path)?.get(key)
}

fn failure(data: &[u8]) -> BinaryError {
    let error = lookup(data, "я").unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<BinaryError>()
            .is_some()
    );
    let DictionaryError::BinaryError { path, source } = error else {
        panic!("unexpected error: {error:?}")
    };
    assert_eq!(path.file_name().unwrap(), "words.dawg");
    source
}

#[test]
fn payload_is_big_endian_and_trailing_newline_is_optional() {
    for payload in [b"AQIDBA==".as_slice(), b"AQIDBA==\n"] {
        let fixture = DawgFixture::new("я", payload);
        assert_eq!(
            lookup(&fixture.bytes(), "я").unwrap(),
            [WordRecord {
                paradigm_id: 258,
                form_index: 772
            }]
        );
        for missing in ["Я", "а", "яя", "", "\0", "я\u{1}"] {
            assert!(
                lookup(&fixture.bytes(), missing).unwrap().is_empty(),
                "{missing:?}"
            );
        }
    }
}

#[test]
fn invalid_sizes_and_truncated_or_trailing_data_fail_loading() {
    let data = DawgFixture::new("я", b"AQIDBA==\n").bytes();
    for end in [0, 1, 3, 4, 8, 256, data.len() - 1] {
        assert!(lookup(&data[..end], "я").is_err(), "length {end}");
    }
    assert!(matches!(failure(&[0, 0, 0, 0]), BinaryError::EmptyDawg));
    assert!(matches!(
        failure(&[255; 4]),
        BinaryError::CountMismatch { .. }
    ));
    let mut extra = data.clone();
    extra.push(0);
    assert!(matches!(
        failure(&extra),
        BinaryError::TrailingData { remaining: 1, .. }
    ));
    let mut wrong_guide = DawgFixture::new("я", b"AQIDBA==\n");
    wrong_guide.guide.pop();
    assert!(matches!(
        failure(&wrong_guide.bytes()),
        BinaryError::CountMismatch { .. }
    ));
}

#[test]
fn corrupted_offsets_guides_and_leaf_references_return_errors() {
    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    data.units[0] = 0xffff_fc00;
    assert!(matches!(
        failure(&data.bytes()),
        BinaryError::UnitOutOfBounds { .. }
    ));

    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    let payload_root = data.path[3];
    data.guide[payload_root][0] = b'!';
    assert!(matches!(
        failure(&data.bytes()),
        BinaryError::InvalidTransition { label: b'!', .. }
    ));

    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    data.guide[data.path[3]][0] = 0;
    assert!(matches!(
        failure(&data.bytes()),
        BinaryError::MissingTerminal { .. }
    ));

    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    let last = *data.path.last().unwrap();
    data.guide[last][1] = b'\n';
    assert!(matches!(
        failure(&data.bytes()),
        BinaryError::RepeatedSibling { label: b'\n', .. }
    ));

    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    let value_index = data.units.len() - 256;
    data.units[value_index] = 0;
    assert!(matches!(
        failure(&data.bytes()),
        BinaryError::InvalidLeaf { .. }
    ));
}

#[test]
fn payload_size_encoding_and_record_width_are_checked() {
    assert!(matches!(
        failure(&DawgFixture::new("я", b"AAAAAAAAAAAA").bytes()),
        BinaryError::PayloadTooLong { limit: 9 }
    ));
    let error = failure(&DawgFixture::new("я", b"!!!!!!!!").bytes());
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<base64::DecodeError>()
            .is_some()
    );
    assert!(matches!(error, BinaryError::InvalidBase64 { .. }));
    assert!(matches!(
        failure(&DawgFixture::new("я", b"AA==").bytes()),
        BinaryError::InvalidRecordLength {
            expected: 4,
            actual: 1
        }
    ));
}

#[test]
fn prediction_record_frequency_keeps_all_four_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("prediction.dawg");
    fs::write(&path, DawgFixture::new("я", b"AQIDBAUGBwg=\n").bytes()).unwrap();
    let reader = RecordDawg::<PredictionRecord>::load(path).unwrap();
    assert_eq!(
        reader.get("я").unwrap(),
        [PredictionRecord {
            count: 0x01020304,
            paradigm_id: 0x0506,
            form_index: 0x0708
        }]
    );
}

#[test]
fn extended_offset_encoding_follows_the_same_transition() {
    let mut data = DawgFixture::new("я", b"AQIDBA==\n");
    data.units[0] = (1 << 10) | (1 << 9);
    assert_eq!(
        lookup(&data.bytes(), "я").unwrap(),
        [WordRecord {
            paradigm_id: 258,
            form_index: 772
        }]
    );
}
