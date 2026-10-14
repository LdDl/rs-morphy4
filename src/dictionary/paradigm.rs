use std::path::Path;

use crate::dictionary::binary::cursor::Cursor;
use crate::dictionary::binary::error::{BinaryError, TableKind};
use crate::dictionary::error::DictionaryError;
use crate::dictionary::file;
use crate::dictionary::tables::DictionaryTables;

/// Indices of one form in the ordered dictionary tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParadigmForm {
    pub suffix_id: u16,
    pub tag_id: u16,
    pub prefix_id: u16,
}

#[derive(Debug)]
pub struct Paradigm {
    forms: Vec<ParadigmForm>,
}

impl Paradigm {
    pub fn forms(&self) -> &[ParadigmForm] {
        &self.forms
    }
}

#[derive(Debug)]
pub struct Paradigms {
    entries: Vec<Paradigm>,
}

impl Paradigms {
    /// Loads little-endian arrays and validates all references against the tables.
    pub fn load(
        path: impl AsRef<Path>,
        tables: &DictionaryTables,
    ) -> Result<Self, DictionaryError> {
        let path = path.as_ref();
        let bytes = file::read(path)?;
        Self::decode(&bytes, tables).map_err(|source| DictionaryError::binary(path, source))
    }

    pub fn entries(&self) -> &[Paradigm] {
        &self.entries
    }

    fn decode(bytes: &[u8], tables: &DictionaryTables) -> Result<Self, BinaryError> {
        let mut cursor = Cursor::new(bytes);
        let count = usize::from(cursor.u16()?);
        if count != tables.metadata.paradigms_length {
            return Err(BinaryError::CountMismatch {
                expected: tables.metadata.paradigms_length,
                actual: count,
            });
        }
        let mut entries = Vec::with_capacity(count);
        for paradigm in 0..count {
            let length = usize::from(cursor.u16()?);
            if length == 0 || length % 3 != 0 {
                return Err(BinaryError::InvalidParadigmLength { paradigm, length });
            }
            let data = cursor.take(length * 2)?;
            let ids: Vec<u16> = data
                .as_chunks::<2>()
                .0
                .iter()
                .copied()
                .map(u16::from_le_bytes)
                .collect();
            let n = length / 3;
            let mut forms = Vec::with_capacity(n);
            for form in 0..n {
                let entry = ParadigmForm {
                    suffix_id: ids[form],
                    tag_id: ids[n + form],
                    prefix_id: ids[n * 2 + form],
                };
                for (table, index, length) in [
                    (TableKind::Suffix, entry.suffix_id, tables.suffixes.len()),
                    (TableKind::Tag, entry.tag_id, tables.tags.len()),
                    (
                        TableKind::Prefix,
                        entry.prefix_id,
                        tables.metadata.compile_options.paradigm_prefixes.len(),
                    ),
                ] {
                    if usize::from(index) >= length {
                        return Err(BinaryError::InvalidReference {
                            table,
                            index: usize::from(index),
                            length,
                            paradigm,
                            form,
                        });
                    }
                }
                forms.push(entry);
            }
            entries.push(Paradigm { forms });
        }
        cursor.finish()?;
        Ok(Self { entries })
    }
}
