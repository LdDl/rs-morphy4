use crate::analysis::context::types::{
    CandidateSupport, ContextDecision, ContextReading, ContextStatus,
};
use crate::analysis::parse::Parse;

pub fn decide(candidates: &[Parse], support: Vec<CandidateSupport>) -> ContextDecision {
    let maximum = support.iter().map(|row| row.support).max().unwrap_or(0);
    let mut retained: Vec<ContextReading> = Vec::new();
    for row in support.iter().filter(|row| row.support == maximum) {
        let parse = &candidates[row.index];
        if let Some(reading) = retained
            .iter_mut()
            .find(|reading| reading.lemma == parse.normal_form && reading.pos == parse.tag.pos)
        {
            reading.parse_indices.push(row.index);
        } else {
            retained.push(ContextReading {
                lemma: parse.normal_form.clone(),
                pos: parse.tag.pos.clone(),
                parse_indices: vec![row.index],
            });
        }
    }
    let selected = if retained.len() == 1 {
        retained.first().cloned()
    } else {
        None
    };
    let status = if candidates.is_empty() {
        ContextStatus::NoCandidates
    } else if selected.is_some() {
        if maximum > 0 {
            ContextStatus::SupportedReading
        } else {
            ContextStatus::OnlyReading
        }
    } else if maximum > 0 {
        ContextStatus::Ambiguous
    } else {
        ContextStatus::NoEvidence
    };
    ContextDecision {
        status,
        baseline_index: (!candidates.is_empty()).then_some(0),
        selected,
        retained,
        support,
    }
}
