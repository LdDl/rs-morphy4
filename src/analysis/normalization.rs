use std::collections::HashSet;

use crate::analysis::dictionary::DictionaryAnalyzer;
use crate::analysis::parse::Parse;
use crate::dictionary::DictionaryError;

impl DictionaryAnalyzer {
    /// Lowercases and trims input, then returns dictionary analyses without ranking.
    ///
    /// An alternate spelling swaps all е/ё simultaneously. It is checked for words
    /// of at most six characters or when the original spelling is absent.
    /// Mixed replacement combinations are not generated. Unknown words return no analyses.
    pub fn parse(&self, word: &str) -> Result<Vec<Parse>, DictionaryError> {
        let word = word
            .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
            .to_lowercase();
        if word.is_empty() {
            return Ok(Vec::new());
        }
        let original = self.parse_exact(&word)?;
        if !word.contains(['е', 'ё']) || (!original.is_empty() && word.chars().count() > 6) {
            return Ok(original);
        }
        let alternate: String = word
            .chars()
            .map(|c| match c {
                'е' => 'ё',
                'ё' => 'е',
                _ => c,
            })
            .collect();
        let mut extra = self.parse_exact(&alternate)?;
        if extra.is_empty() {
            return Ok(original);
        }
        let original_keys: HashSet<_> = original.iter().map(|p| (&p.normal_form, &p.tag)).collect();
        let extra_keys: HashSet<_> = extra.iter().map(|p| (&p.normal_form, &p.tag)).collect();
        if original_keys == extra_keys {
            return Ok(original);
        }
        for parse in &mut extra {
            parse.word.clone_from(&word);
        }
        let mut seen = HashSet::new();
        Ok(original
            .into_iter()
            .chain(extra)
            .filter(|p| seen.insert((p.normal_form.clone(), p.tag.clone())))
            .collect())
    }
}
