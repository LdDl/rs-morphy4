use std::collections::BTreeSet;
use std::fmt;

/// A part of speech and a set of grammatical features.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub pos: String,
    pub grammemes: BTreeSet<String>,
}

impl Tag {
    /// Creates a tag without validating or normalizing its features.
    pub fn new<I, S>(pos: impl Into<String>, grammemes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            pos: pos.into(),
            grammemes: grammemes.into_iter().map(Into::into).collect(),
        }
    }

    /// Reports membership in the feature set or equality with the part of speech.
    pub fn contains(&self, grammeme: &str) -> bool {
        self.pos == grammeme || self.grammemes.contains(grammeme)
    }

    /// Returns a feature set with rare cases replaced by their common forms.
    pub fn fix_rare_cases(grammemes: &BTreeSet<String>) -> BTreeSet<String> {
        grammemes
            .iter()
            .map(|grammeme| {
                match grammeme.as_str() {
                    "gen1" | "gen2" => "gent",
                    "acc1" | "acc2" => "accs",
                    "loc1" | "loc2" => "loct",
                    "voct" => "nomn",
                    other => other,
                }
                .to_owned()
            })
            .collect()
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            pos: "UNKN".to_owned(),
            grammemes: BTreeSet::new(),
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pos)?;
        for (index, grammeme) in self.grammemes.iter().enumerate() {
            f.write_str(if index == 0 { " " } else { "," })?;
            f.write_str(grammeme)?;
        }
        Ok(())
    }
}
