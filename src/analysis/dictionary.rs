use std::path::{Path, PathBuf};

use crate::analysis::parse::Parse;
use crate::dictionary::dawg::{RecordDawg, WordRecord};
use crate::dictionary::{
    BinaryError, DictionaryData, DictionaryError, DictionaryTables, Paradigms, TableKind,
};

#[derive(Debug)]
/// In-memory dictionary lookup without prediction or special-token rules.
pub struct DictionaryAnalyzer {
    tables: DictionaryTables,
    paradigms: Paradigms,
    words: RecordDawg<WordRecord>,
    words_path: PathBuf,
}

impl DictionaryAnalyzer {
    /// Loads the bundled Russian dictionary entirely from embedded bytes.
    /// Invalid bundled tables or binary structures return a dictionary error.
    pub fn embedded() -> Result<Self, DictionaryError> {
        Self::from_data(DictionaryData::bundled())
    }

    /// Decodes a caller-provided dictionary without filesystem access.
    /// Malformed or unsupported data returns a typed error; input bytes are not retained.
    pub fn from_data(data: DictionaryData<'_>) -> Result<Self, DictionaryError> {
        let tables = DictionaryTables::from_data(data)?;
        let paradigms = Paradigms::from_bytes(data.paradigms, &tables, "<memory>/paradigms.array")?;
        let words_path = PathBuf::from("<memory>/words.dawg");
        let words = RecordDawg::from_bytes(data.words, &words_path)?;
        Ok(Self {
            tables,
            paradigms,
            words,
            words_path,
        })
    }

    /// Loads JSON tables, paradigms and words. Prediction files are not required.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, DictionaryError> {
        let directory = directory.as_ref();
        let tables = DictionaryTables::load(directory)?;
        let paradigms = Paradigms::load(directory.join("paradigms.array"), &tables)?;
        let words_path = directory.join("words.dawg");
        let words = RecordDawg::load(&words_path)?;
        Ok(Self {
            tables,
            paradigms,
            words,
            words_path,
        })
    }

    /// Borrows the ordered JSON tables and compilation metadata.
    pub fn tables(&self) -> &DictionaryTables {
        &self.tables
    }

    /// Borrows the validated inflection paradigms.
    pub fn paradigms(&self) -> &Paradigms {
        &self.paradigms
    }

    /// Parses an exact, case-sensitive dictionary spelling without ranking or deduplication.
    pub fn parse_exact(&self, word: &str) -> Result<Vec<Parse>, DictionaryError> {
        self.words
            .get(word)?
            .into_iter()
            .map(|record| {
                self.resolve(word, record)
                    .map_err(|source| DictionaryError::binary(&self.words_path, source))
            })
            .collect()
    }

    /// Resolves stored spellings with independent е -> ё substitutions.
    /// Input case is preserved; explicit ё is never replaced by е.
    pub fn parse_with_yo(&self, word: &str) -> Result<Vec<Parse>, DictionaryError> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for matched in self.words.get_with_yo(word)? {
            for record in matched.records {
                let mut parse = self
                    .resolve(&matched.word, record)
                    .map_err(|source| DictionaryError::binary(&self.words_path, source))?;
                parse.word = word.to_owned();
                if seen.insert((parse.normal_form.clone(), parse.tag.clone())) {
                    result.push(parse);
                }
            }
        }
        Ok(result)
    }

    fn resolve(&self, word: &str, record: WordRecord) -> Result<Parse, BinaryError> {
        let paradigm = usize::from(record.paradigm_id);
        let form = usize::from(record.form_index);
        let entries = self.paradigms.entries();
        let forms = entries
            .get(paradigm)
            .ok_or(BinaryError::InvalidReference {
                table: TableKind::Paradigm,
                index: paradigm,
                length: entries.len(),
                paradigm,
                form,
            })?
            .forms();
        let current = forms.get(form).ok_or(BinaryError::InvalidReference {
            table: TableKind::Form,
            index: form,
            length: forms.len(),
            paradigm,
            form,
        })?;
        let prefixes = &self.tables.metadata.compile_options.paradigm_prefixes;
        let suffixes = &self.tables.suffixes;
        let prefix = &prefixes[usize::from(current.prefix_id)];
        let suffix = &suffixes[usize::from(current.suffix_id)];
        let stem = word
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(suffix))
            .ok_or(BinaryError::AffixMismatch { paradigm, form })?;
        // Loading guarantees a nonempty paradigm and valid table references.
        let normal = forms[0];
        let normal_form = format!(
            "{}{}{}",
            prefixes[usize::from(normal.prefix_id)],
            stem,
            suffixes[usize::from(normal.suffix_id)]
        );
        Ok(Parse {
            word: word.to_owned(),
            normal_form,
            tag: self.tables.tags[usize::from(current.tag_id)].clone(),
            score: 1.0,
            paradigm_id: Some(record.paradigm_id),
            form_index: Some(record.form_index),
            stem: Some(stem.to_owned()),
        })
    }
}
