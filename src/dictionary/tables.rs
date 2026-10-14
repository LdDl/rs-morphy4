use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::tag::Tag;

use crate::dictionary::error::{DictionaryError, ValidationError};
use crate::dictionary::metadata::{Grammeme, Metadata};

/// Ordered tables whose indices are used by binary paradigms and DAWGs.
#[derive(Debug)]
pub struct DictionaryTables {
    pub metadata: Metadata,
    pub grammemes: Vec<Grammeme>,
    pub tags: Vec<Tag>,
    pub suffixes: Vec<String>,
}

impl DictionaryTables {
    /// Loads the JSON tables from a directory containing a Russian 2.4 dictionary.
    ///
    /// Binary paradigms and DAWGs are not read by this method.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        let directory = directory.as_ref();
        let metadata_path = directory.join("meta.json");
        let metadata = read_metadata(&metadata_path)?;
        let gramtab_name = metadata
            .gramtab_formats
            .get("opencorpora-int")
            .ok_or_else(|| {
                DictionaryError::invalid(
                    &metadata_path,
                    ValidationError::MissingTable {
                        format: "opencorpora-int".into(),
                    },
                )
            })?;
        let mut components = Path::new(gramtab_name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(DictionaryError::invalid(
                &metadata_path,
                ValidationError::InvalidTableFilename {
                    filename: gramtab_name.clone(),
                },
            ));
        }

        let grammemes_path = directory.join("grammemes.json");
        let grammemes: Vec<Grammeme> = read_json(&grammemes_path)?;
        let mut names = BTreeSet::new();
        for grammeme in &grammemes {
            if grammeme.name.is_empty() {
                return Err(DictionaryError::invalid(
                    &grammemes_path,
                    ValidationError::EmptyGrammemeName,
                ));
            }
            if !names.insert(grammeme.name.as_str()) {
                return Err(DictionaryError::invalid(
                    &grammemes_path,
                    ValidationError::DuplicateGrammemeName {
                        name: grammeme.name.clone(),
                    },
                ));
            }
        }
        for grammeme in &grammemes {
            if !grammeme.parent.is_empty() && !names.contains(grammeme.parent.as_str()) {
                return Err(DictionaryError::invalid(
                    &grammemes_path,
                    ValidationError::UnknownParent {
                        name: grammeme.name.clone(),
                        parent: grammeme.parent.clone(),
                    },
                ));
            }
        }

        let gramtab_path = directory.join(gramtab_name);
        let entries: Vec<String> = read_json(&gramtab_path)?;
        check_length(&gramtab_path, entries.len(), metadata.gramtab_length)?;
        let mut tags = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let mut parts = entry.split([' ', ',']);
            let pos = parts.next().unwrap_or("");
            let tag = Tag::new(pos, parts);
            for name in
                std::iter::once(tag.pos.as_str()).chain(tag.grammemes.iter().map(String::as_str))
            {
                if !names.contains(name) {
                    return Err(DictionaryError::invalid(
                        &gramtab_path,
                        ValidationError::UnknownGrammeme {
                            name: name.to_owned(),
                            tag_index: index,
                        },
                    ));
                }
            }
            tags.push(tag);
        }

        let suffixes_path = directory.join("suffixes.json");
        let suffixes: Vec<String> = read_json(&suffixes_path)?;
        check_length(&suffixes_path, suffixes.len(), metadata.suffixes_length)?;
        Ok(Self {
            metadata,
            grammemes,
            tags,
            suffixes,
        })
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, DictionaryError> {
    let content = fs::read_to_string(path).map_err(|source| DictionaryError::IoError {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| DictionaryError::ParseError {
        path: path.to_owned(),
        source,
    })
}

fn read_metadata(path: &Path) -> Result<Metadata, DictionaryError> {
    let pairs: Vec<(String, Value)> = read_json(path)?;
    let mut fields = Map::new();
    for (key, value) in pairs {
        if fields.insert(key.clone(), value).is_some() {
            return Err(DictionaryError::invalid(
                path,
                ValidationError::DuplicateMetadataKey { key },
            ));
        }
    }
    let metadata: Metadata = serde_json::from_value(Value::Object(fields)).map_err(|source| {
        DictionaryError::ParseError {
            path: path.to_owned(),
            source,
        }
    })?;
    if metadata.format_version != "2.4" {
        return Err(DictionaryError::invalid(
            path,
            ValidationError::UnsupportedFormat {
                found: metadata.format_version,
                expected: "2.4".into(),
            },
        ));
    }
    if metadata.language_code != "ru" {
        return Err(DictionaryError::invalid(
            path,
            ValidationError::UnsupportedLanguage {
                found: metadata.language_code,
                expected: "ru".into(),
            },
        ));
    }
    if metadata.compile_options.paradigm_prefixes.is_empty() {
        return Err(DictionaryError::invalid(
            path,
            ValidationError::EmptyParadigmPrefixes,
        ));
    }
    check_length(
        path,
        metadata.prediction_suffixes_dawg_lengths.len(),
        metadata.compile_options.paradigm_prefixes.len(),
    )?;
    Ok(metadata)
}

fn check_length(path: &Path, actual: usize, expected: usize) -> Result<(), DictionaryError> {
    if actual != expected {
        return Err(DictionaryError::invalid(
            path,
            ValidationError::LengthMismatch { expected, actual },
        ));
    }
    Ok(())
}
