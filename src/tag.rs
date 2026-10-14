use std::collections::BTreeSet;
use std::fmt;

/// A part of speech and a set of grammatical features.
///
/// Feature accessors return the lexicographically first recognized value when
/// the set contains conflicting values of the same category.
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

    /// Returns the grammatical case without normalizing rare cases.
    pub fn case(&self) -> Option<&str> {
        self.feature(&["nomn", "gent", "datv", "accs", "ablt", "loct", "voct"])
    }

    /// Returns singular or plural number.
    pub fn number(&self) -> Option<&str> {
        self.feature(&["sing", "plur"])
    }

    /// Returns masculine, feminine or neuter gender.
    pub fn gender(&self) -> Option<&str> {
        self.feature(&["masc", "femn", "neut"])
    }

    /// Returns perfective or imperfective aspect.
    pub fn aspect(&self) -> Option<&str> {
        self.feature(&["perf", "impf"])
    }

    /// Returns past, present or future tense.
    pub fn tense(&self) -> Option<&str> {
        self.feature(&["past", "pres", "futr"])
    }

    fn feature(&self, values: &[&str]) -> Option<&str> {
        self.grammemes
            .iter()
            .map(String::as_str)
            .find(|value| values.contains(value))
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
