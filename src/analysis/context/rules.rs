use crate::analysis::context::agreement::{
    all_pos, has_any, nominal_agreement, preposition_cases, subject_agreement,
};
use crate::analysis::context::constructions::{
    imperative_with_object, infinitive_governed, inverted_subject,
};
use crate::analysis::context::phrases::{locative_existence, numeral_agreement};
use crate::analysis::context::types::{CandidateSupport, ContextRule};
use crate::analysis::parse::Parse;

pub fn evaluate(
    tokens: &[String],
    candidates: &[Vec<Parse>],
    target: usize,
) -> Vec<CandidateSupport> {
    let left = target
        .checked_sub(1)
        .map(|i| candidates[i].as_slice())
        .unwrap_or_default();
    let right = candidates
        .get(target + 1)
        .map(Vec::as_slice)
        .unwrap_or_default();
    candidates[target]
        .iter()
        .enumerate()
        .map(|(index, parse)| {
            let mut evidence = Vec::new();
            let pos = parse.tag.pos.as_str();
            if pos == "NOUN"
                && all_pos(left, &["ADJF", "PRTF"])
                && left.iter().any(|p| nominal_agreement(p, parse))
            {
                evidence.push(ContextRule::LeftModifierAgreement);
            }
            if ["ADJF", "PRTF"].contains(&pos)
                && all_pos(right, &["NOUN"])
                && right.iter().any(|p| nominal_agreement(parse, p))
            {
                evidence.push(ContextRule::RightNounAgreement);
            }
            if pos == "VERB"
                && all_pos(left, &["NOUN", "NPRO"])
                && left.iter().any(|p| subject_agreement(p, parse))
            {
                evidence.push(ContextRule::LeftSubjectAgreement);
            }
            if pos == "VERB" && all_pos(right, &["INFN"]) {
                evidence.push(ContextRule::FollowingInfinitive);
            }
            if pos == "NOUN" && all_pos(left, &["NOUN"]) && has_any(parse, &["gent", "gen2"]) {
                evidence.push(ContextRule::PossibleGenitiveComplement);
            }
            if target > 0
                && all_pos(left, &["PREP"])
                && ["NOUN", "NPRO", "ADJF", "PRTF"].contains(&pos)
                && has_any(parse, preposition_cases(&tokens[target - 1]))
            {
                evidence.push(ContextRule::PrepositionCase);
            }
            if inverted_subject(tokens, candidates, target, parse) {
                evidence.push(ContextRule::PossibleInvertedSubject);
            }
            if infinitive_governed(tokens, candidates, target, parse) {
                evidence.push(ContextRule::GovernedInfinitive);
            }
            if imperative_with_object(tokens, candidates, target, parse) {
                evidence.push(ContextRule::ImperativeObject);
            }
            if numeral_agreement(candidates, target, parse) {
                evidence.push(ContextRule::NumeralAgreement);
            }
            if locative_existence(tokens, candidates, target, parse) {
                evidence.push(ContextRule::LocativeExistence);
            }
            CandidateSupport {
                index,
                support: evidence.iter().map(|rule| rule.weight()).sum(),
                evidence,
            }
        })
        .collect()
}
