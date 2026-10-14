//! Metadata and JSON tables for compiled dictionaries.

mod error;
mod metadata;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::Tag;

pub use error::DictionaryError;
pub use metadata::{CompileOptions, Grammeme, Metadata};

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
                DictionaryError::invalid(&metadata_path, "missing opencorpora-int table filename")
            })?;
        let mut components = Path::new(gramtab_name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(DictionaryError::invalid(
                &metadata_path,
                "opencorpora-int table must be a filename within the dictionary directory",
            ));
        }

        let grammemes_path = directory.join("grammemes.json");
        let grammemes: Vec<Grammeme> = read_json(&grammemes_path)?;
        let names: BTreeSet<&str> = grammemes.iter().map(|g| g.name.as_str()).collect();
        if names.contains("") || names.len() != grammemes.len() {
            return Err(DictionaryError::invalid(
                &grammemes_path,
                "empty or duplicate grammeme name",
            ));
        }
        for grammeme in &grammemes {
            if !grammeme.parent.is_empty() && !names.contains(grammeme.parent.as_str()) {
                return Err(DictionaryError::invalid(
                    &grammemes_path,
                    format!(
                        "unknown parent {:?} of grammeme {:?}",
                        grammeme.parent, grammeme.name
                    ),
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
                        format!("unknown or empty grammeme {name:?} in tag {index}"),
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
    let content = fs::read_to_string(path).map_err(|error| {
        DictionaryError::IoError(format!(
            "cannot read dictionary file {}: {error}",
            path.display()
        ))
    })?;
    serde_json::from_str(&content).map_err(|error| {
        DictionaryError::ParseError(format!(
            "cannot parse dictionary file {}: {error}",
            path.display()
        ))
    })
}

fn read_metadata(path: &Path) -> Result<Metadata, DictionaryError> {
    let pairs: Vec<(String, Value)> = read_json(path)?;
    let mut fields = Map::new();
    for (key, value) in pairs {
        if fields.insert(key.clone(), value).is_some() {
            return Err(DictionaryError::invalid(
                path,
                format!("duplicate metadata key {key:?}"),
            ));
        }
    }
    let metadata: Metadata = serde_json::from_value(Value::Object(fields)).map_err(|error| {
        DictionaryError::ParseError(format!(
            "cannot parse dictionary file {}: {error}",
            path.display()
        ))
    })?;
    if metadata.format_version != "2.4" {
        return Err(DictionaryError::invalid(
            path,
            format!(
                "unsupported format {:?}; expected 2.4",
                metadata.format_version
            ),
        ));
    }
    if metadata.language_code != "ru" {
        return Err(DictionaryError::invalid(
            path,
            format!(
                "unsupported language {:?}; expected ru",
                metadata.language_code
            ),
        ));
    }
    if metadata.compile_options.paradigm_prefixes.is_empty() {
        return Err(DictionaryError::invalid(
            path,
            "paradigm prefixes are empty",
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
            format!("expected {expected} entries, found {actual}"),
        ));
    }
    Ok(())
}
