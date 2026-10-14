use crate::tag::Tag;

/// One analysis. Score is a rule weight, not a calibrated probability.
#[derive(Debug, Clone, PartialEq)]
pub struct Parse {
    /// Analyzed surface form after input normalization and spelling rules.
    pub word: String,
    /// Lemma associated with this analysis.
    pub normal_form: String,
    /// Part of speech and grammatical features.
    pub tag: Tag,
    /// Rule weight; ranking also considers part of speech and tag features.
    pub score: f64,
    /// Dictionary paradigm used for inflection, when available.
    pub paradigm_id: Option<u16>,
    /// Position of this form within its paradigm, when available.
    pub form_index: Option<u16>,
    /// Stem retained for reconstructing dictionary forms, when available.
    pub stem: Option<String>,
}

impl Parse {
    /// Creates an analysis without a dictionary paradigm or stem.
    pub fn synthetic(word: &str, normal_form: String, tag: Tag, score: f64) -> Self {
        Self {
            word: word.to_owned(),
            normal_form,
            tag,
            score,
            paradigm_id: None,
            form_index: None,
            stem: None,
        }
    }
}
