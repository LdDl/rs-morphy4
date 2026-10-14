use crate::tag::Tag;

/// One analysis. Score is a rule weight, not a calibrated probability.
#[derive(Debug, Clone, PartialEq)]
pub struct Parse {
    pub word: String,
    pub normal_form: String,
    pub tag: Tag,
    pub score: f64,
    pub paradigm_id: Option<u16>,
    pub form_index: Option<u16>,
    pub stem: Option<String>,
}

impl Parse {
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
