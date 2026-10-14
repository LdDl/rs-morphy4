use std::error::Error;
use std::fs;
use std::path::Path;

use rs_morphy4::dictionary::{
    BinaryError, DictionaryError, DictionaryTables, Paradigms, TableKind,
};

fn tables() -> DictionaryTables {
    let mut tables =
        DictionaryTables::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru"))
            .unwrap();
    tables.metadata.paradigms_length = 1;
    tables
}

fn bytes(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|n| n.to_le_bytes()).collect()
}

fn load_error(data: &[u8], tables: &DictionaryTables) -> BinaryError {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("paradigms.array");
    fs::write(&path, data).unwrap();
    let error = Paradigms::load(&path, tables).unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<BinaryError>()
            .is_some()
    );
    let DictionaryError::BinaryError {
        path: actual,
        source,
    } = error
    else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(actual, path);
    source
}

#[test]
fn forms_read_three_separate_index_arrays_in_little_endian() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("paradigms.array");
    fs::write(&path, bytes(&[1, 6, 256, 300, 10, 500, 0, 2])).unwrap();
    let paradigms = Paradigms::load(path, &tables()).unwrap();
    let forms = paradigms.entries()[0].forms();
    assert_eq!(
        (forms[0].suffix_id, forms[0].tag_id, forms[0].prefix_id),
        (256, 10, 0)
    );
    assert_eq!(
        (forms[1].suffix_id, forms[1].tag_id, forms[1].prefix_id),
        (300, 500, 2)
    );
}

#[test]
fn truncated_arrays_and_trailing_bytes_are_rejected() {
    let tables = tables();
    let mut data = bytes(&[1, 3, 0, 0, 0]);
    for end in 0..data.len() {
        assert!(
            matches!(
                load_error(&data[..end], &tables),
                BinaryError::UnexpectedEnd { .. }
            ),
            "length {end}"
        );
    }
    data.push(99);
    assert!(matches!(
        load_error(&data, &tables),
        BinaryError::TrailingData {
            offset: 10,
            remaining: 1
        }
    ));
}

#[test]
fn counts_and_nonempty_tripartite_structure_are_checked() {
    let tables = tables();
    assert!(matches!(
        load_error(&bytes(&[2]), &tables),
        BinaryError::CountMismatch {
            expected: 1,
            actual: 2
        }
    ));
    for length in [0, 1, 2, 4, 5] {
        assert!(
            matches!(load_error(&bytes(&[1, length]), &tables), BinaryError::InvalidParadigmLength { paradigm: 0, length: actual } if actual == usize::from(length))
        );
    }
}

#[test]
fn every_table_reference_is_checked_before_exposing_forms() {
    let tables = tables();
    for (position, table, length) in [
        (2, TableKind::Suffix, tables.suffixes.len()),
        (3, TableKind::Tag, tables.tags.len()),
        (
            4,
            TableKind::Prefix,
            tables.metadata.compile_options.paradigm_prefixes.len(),
        ),
    ] {
        let mut data = [1, 3, 0, 0, 0];
        data[position] = u16::try_from(length).unwrap();
        let BinaryError::InvalidReference {
            table: actual_table,
            index,
            length: actual_length,
            paradigm,
            form,
        } = load_error(&bytes(&data), &tables)
        else {
            panic!("unexpected error")
        };
        assert_eq!(
            (actual_table, index, actual_length, paradigm, form),
            (table, length, length, 0, 0)
        );
    }
}

#[test]
fn missing_binary_file_retains_io_cause() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("absent.array");
    let DictionaryError::IoError {
        path: actual,
        source,
    } = Paradigms::load(&path, &tables()).unwrap_err()
    else {
        panic!("unexpected error")
    };
    assert_eq!(actual, path);
    assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
}
