use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde_json::{Map, Value};

use crate::tag::Tag;

use crate::dictionary::data::DictionaryData;
use crate::dictionary::error::{DictionaryError, ValidationError};
use crate::dictionary::metadata::{Grammeme, Metadata};
use crate::dictionary::source::{JsonTable, Source};

/// Ordered tables whose indices are used by binary paradigms and DAWGs.
#[derive(Debug)]
pub struct DictionaryTables {
    /// Source and compilation settings.
    pub metadata: Metadata,
    /// Feature definitions and their parent categories.
    pub grammemes: Vec<Grammeme>,
    /// Tags in the identifier order used by paradigms.
    pub tags: Vec<Tag>,
    /// Suffix strings in the identifier order used by paradigms.
    pub suffixes: Vec<String>,
}

impl DictionaryTables {
    /// Loads the JSON tables from a directory containing a Russian 2.4 dictionary.
    ///
    /// Binary paradigms and DAWGs are not read by this method.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        Self::load_from(Source::Directory(directory.as_ref()))
    }

    /// Decodes and validates the bundled JSON tables without filesystem access.
    pub fn embedded() -> Result<Self, DictionaryError> {
        Self::from_data(DictionaryData::bundled())
    }

    /// Decodes caller-provided JSON tables using the same validation as directory loading.
    pub fn from_data(data: DictionaryData<'_>) -> Result<Self, DictionaryError> {
        Self::load_from(Source::Memory(data))
    }

    fn load_from(source: Source<'_>) -> Result<Self, DictionaryError> {
        let metadata_path = source.path("meta.json");
        let metadata = read_metadata(&source, &metadata_path)?;
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

        let grammemes_path = source.path("grammemes.json");
        let grammemes: Vec<Grammeme> = source.json(&grammemes_path, JsonTable::Grammemes)?;
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

        let gramtab_path = source.path(gramtab_name);
        let entries: Vec<String> = source.json(&gramtab_path, JsonTable::Tags)?;
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

        let suffixes_path = source.path("suffixes.json");
        let suffixes: Vec<String> = source.json(&suffixes_path, JsonTable::Suffixes)?;
        check_length(&suffixes_path, suffixes.len(), metadata.suffixes_length)?;
        Ok(Self {
            metadata,
            grammemes,
            tags,
            suffixes,
        })
    }
}

fn read_metadata(source: &Source<'_>, path: &Path) -> Result<Metadata, DictionaryError> {
    let pairs: Vec<(String, Value)> = source.json(path, JsonTable::Metadata)?;
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
