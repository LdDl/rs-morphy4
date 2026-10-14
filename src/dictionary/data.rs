use crate::dictionary::embedded;

/// Borrowed files of a compiled Russian dictionary in format 2.4.
/// Loaders decode owned structures; these slices need only live through construction.
#[derive(Debug, Clone, Copy)]
pub struct DictionaryData<'a> {
    /// Contents of meta.json.
    pub metadata: &'a [u8],
    /// Contents of grammemes.json.
    pub grammemes: &'a [u8],
    /// Contents of the opencorpora-int tag table named in metadata.
    pub tags: &'a [u8],
    /// Contents of suffixes.json.
    pub suffixes: &'a [u8],
    /// Contents of paradigms.array.
    pub paradigms: &'a [u8],
    /// Contents of words.dawg.
    pub words: &'a [u8],
}

impl DictionaryData<'static> {
    /// Returns the bundled Russian dictionary bytes without decoding or filesystem access.
    pub fn bundled() -> Self {
        Self {
            metadata: embedded::METADATA,
            grammemes: embedded::GRAMMEMES,
            tags: embedded::TAGS,
            suffixes: embedded::SUFFIXES,
            paradigms: embedded::PARADIGMS,
            words: embedded::WORDS,
        }
    }
}
