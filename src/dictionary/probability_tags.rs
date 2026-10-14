use std::{collections::HashMap, path::Path};

use crate::dictionary::error::{DictionaryError, ValidationError};
use crate::tag::Tag;

pub fn decode(bytes: &[u8], path: &Path) -> Result<HashMap<Tag, Vec<String>>, DictionaryError> {
    let rows: Vec<String> =
        serde_json::from_slice(bytes).map_err(|source| DictionaryError::ParseError {
            path: path.to_owned(),
            source,
        })?;
    let mut result: HashMap<Tag, Vec<String>> = HashMap::new();
    for (index, row) in rows.into_iter().enumerate() {
        let features: Vec<_> = row
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();
        if features.is_empty() || row.contains([':', '\0']) {
            return Err(DictionaryError::invalid(
                path,
                ValidationError::InvalidProbabilityTag { index },
            ));
        }
        let tag = Tag::new(features[0], features[1..].iter().copied());
        let aliases = result.entry(tag).or_default();
        if !aliases.contains(&row) {
            aliases.push(row);
        }
    }
    Ok(result)
}
