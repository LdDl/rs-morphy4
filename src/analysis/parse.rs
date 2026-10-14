use crate::tag::Tag;

/// One dictionary analysis. Score 1.0 indicates dictionary membership, not probability.
#[derive(Debug, Clone, PartialEq)]
pub struct Parse {
    pub word: String,
    pub normal_form: String,
    pub tag: Tag,
    pub score: f64,
    pub paradigm_id: u16,
    pub form_index: u16,
    pub stem: String,
}
