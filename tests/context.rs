use std::sync::OnceLock;

use rs_morphy4::{ContextDecision, ContextReading, ContextRule, ContextStatus, MorphAnalyzer};
use serde::Deserialize;
use serde_json::{Value, json};

fn analyzer() -> &'static MorphAnalyzer {
    static ANALYZER: OnceLock<MorphAnalyzer> = OnceLock::new();
    ANALYZER.get_or_init(MorphAnalyzer::default)
}

#[derive(Deserialize)]
struct ContextCase {
    id: String,
    tokens: Vec<String>,
    target_index: usize,
    decision: Value,
}

fn status_name(status: ContextStatus) -> &'static str {
    match status {
        ContextStatus::NoCandidates => "no_candidates",
        ContextStatus::OnlyReading => "only_reading",
        ContextStatus::SupportedReading => "supported_reading",
        ContextStatus::Ambiguous => "ambiguous",
        ContextStatus::NoEvidence => "no_evidence",
    }
}

fn rule_name(rule: ContextRule) -> &'static str {
    match rule {
        ContextRule::LeftModifierAgreement => "left_modifier_agreement",
        ContextRule::RightNounAgreement => "right_noun_agreement",
        ContextRule::LeftSubjectAgreement => "left_subject_agreement",
        ContextRule::FollowingInfinitive => "following_infinitive",
        ContextRule::PossibleGenitiveComplement => "possible_genitive_complement",
        ContextRule::PrepositionCase => "preposition_case",
        ContextRule::PossibleInvertedSubject => "possible_inverted_subject",
        ContextRule::GovernedInfinitive => "governed_infinitive",
        ContextRule::ImperativeObject => "imperative_object",
        ContextRule::NumeralAgreement => "numeral_agreement",
        ContextRule::LocativeExistence => "locative_existence",
    }
}

fn reading_json(reading: &ContextReading) -> Value {
    json!({"lemma":reading.lemma, "pos":reading.pos, "parse_indices":reading.parse_indices})
}

fn decision_json(decision: &ContextDecision) -> Value {
    let retained: Vec<_> = decision.retained.iter().map(reading_json).collect();
    let support: Vec<_> = decision
        .support
        .iter()
        .map(|row| {
            let evidence: Vec<_> = row
                .evidence
                .iter()
                .map(|rule| json!({"rule":rule_name(*rule), "weight":rule.weight()}))
                .collect();
            json!({"index":row.index, "support":row.support, "evidence":evidence})
        })
        .collect();
    json!({
        "status":status_name(decision.status), "baseline_index":decision.baseline_index,
        "selected":decision.selected.as_ref().map(reading_json), "retained":retained, "support":support,
    })
}

#[test]
fn context_regressions_preserve_decisions_evidence_and_original_candidates() {
    let cases: Vec<ContextCase> =
        serde_json::from_str(include_str!("fixtures/context.json")).unwrap();
    assert_eq!(cases.len(), 84);
    for case in cases {
        let result = analyzer().parse_context(&case.tokens).unwrap();
        assert_eq!(result.len(), case.tokens.len());
        assert_eq!(
            decision_json(&result[case.target_index].decision),
            case.decision,
            "{}",
            case.id
        );
        for (token, analysis) in case.tokens.iter().zip(result) {
            assert_eq!(analysis.token, *token);
            assert_eq!(
                analysis.candidates,
                analyzer().parse(token).unwrap(),
                "{}: {token}",
                case.id
            );
            for support in &analysis.decision.support {
                assert_eq!(
                    support.support,
                    support
                        .evidence
                        .iter()
                        .map(|rule| rule.weight())
                        .sum::<u32>()
                );
            }
        }
    }
}

#[test]
fn empty_input_preserves_token_positions_without_a_baseline() {
    assert!(
        analyzer()
            .parse_context(Vec::<String>::new())
            .unwrap()
            .is_empty()
    );
    let result = analyzer().parse_context([" ", "", "мы"]).unwrap();
    assert_eq!(result.len(), 3);
    for analysis in &result[..2] {
        assert!(analysis.candidates.is_empty());
        assert_eq!(analysis.decision.status, ContextStatus::NoCandidates);
        assert_eq!(analysis.decision.baseline_index, None);
        assert!(analysis.decision.selected.is_none());
        assert!(analysis.decision.retained.is_empty());
        assert!(analysis.decision.support.is_empty());
    }
    assert!(!result[2].candidates.is_empty());
}

#[test]
fn negation_and_case_normalization_preserve_future_construction() {
    let result = analyzer()
        .parse_context([" Мы ", "БУДЕМ", " НЕ ", "Вести"])
        .unwrap();
    assert_eq!(result[0].token, " Мы ");
    let selected = result[3].decision.selected.as_ref().unwrap();
    assert_eq!(
        (selected.lemma.as_str(), selected.pos.as_str()),
        ("вести", "INFN")
    );
    assert!(
        result[3]
            .decision
            .support
            .iter()
            .any(|row| row.evidence.contains(&ContextRule::GovernedInfinitive))
    );
}

#[test]
fn punctuation_and_empty_tokens_break_governor_links() {
    for boundary in [",", ".", "!", "?", ";", ":", "", " "] {
        let result = analyzer()
            .parse_context(["мы", "будем", boundary, "вести"])
            .unwrap();
        assert!(
            !result[3]
                .decision
                .support
                .iter()
                .any(|row| row.evidence.contains(&ContextRule::GovernedInfinitive)),
            "{boundary:?}"
        );
        assert!(result[3].decision.selected.is_none());
    }
}

#[test]
fn modal_construction_supports_infinitive() {
    let result = analyzer().parse_context(["мы", "хотим", "вести"]).unwrap();
    assert_eq!(result[2].decision.selected.as_ref().unwrap().pos, "INFN");
}

#[test]
fn ambiguity_retains_verbs_without_choosing_the_baseline_noun() {
    let result = analyzer().parse_context(["я", "плачу"]).unwrap();
    let analysis = &result[1];
    assert_eq!(analysis.decision.status, ContextStatus::Ambiguous);
    assert!(analysis.decision.selected.is_none());
    assert_eq!(
        analysis.candidates[analysis.decision.baseline_index.unwrap()]
            .tag
            .pos,
        "NOUN"
    );
    let mut lemmas: Vec<_> = analysis
        .decision
        .retained
        .iter()
        .map(|r| r.lemma.as_str())
        .collect();
    lemmas.sort();
    assert_eq!(lemmas, ["плакать", "платить"]);
}

#[test]
fn one_reading_can_contain_multiple_grammatical_analyses() {
    let result = analyzer().parse_context(["слова"]).unwrap();
    let decision = &result[0].decision;
    assert_eq!(decision.status, ContextStatus::OnlyReading);
    let selected = decision.selected.as_ref().unwrap();
    assert_eq!(selected.lemma, "слово");
    assert!(selected.parse_indices.len() > 1);
    assert_eq!(selected.parse_indices.len(), result[0].candidates.len());
}

#[test]
fn preposition_support_uses_case_and_preserves_all_candidates() {
    let result = analyzer().parse_context(["без", "стали"]).unwrap();
    assert!(
        result[1]
            .decision
            .support
            .iter()
            .any(|row| row.evidence.contains(&ContextRule::PrepositionCase))
    );
    let selected = result[1].decision.selected.as_ref().unwrap();
    assert_eq!(
        (selected.lemma.as_str(), selected.pos.as_str()),
        ("сталь", "NOUN")
    );
    for index in &selected.parse_indices {
        assert!(result[1].candidates[*index].tag.contains("gent"));
    }
}

#[test]
fn ambiguous_preposition_does_not_establish_case_government() {
    for preposition in ["к", "из"] {
        let result = analyzer().parse_context([preposition, "стали"]).unwrap();
        assert!(
            result[0]
                .candidates
                .iter()
                .any(|parse| parse.tag.pos != "PREP")
        );
        assert!(
            !result[1]
                .decision
                .support
                .iter()
                .any(|row| row.evidence.contains(&ContextRule::PrepositionCase))
        );
        assert!(result[1].decision.selected.is_none());
    }
}

#[test]
fn imperative_requires_a_boundary_and_an_unambiguous_accusative_object() {
    let result = analyzer().parse_context(["не", "мой", "чашку"]).unwrap();
    assert_eq!(result[1].decision.selected.as_ref().unwrap().lemma, "мыть");
    for tokens in [vec!["он", "мой", "чашку"], vec!["мой", "чашки"]] {
        let result = analyzer().parse_context(&tokens).unwrap();
        let target = tokens.iter().position(|t| *t == "мой").unwrap();
        assert!(
            !result[target]
                .decision
                .support
                .iter()
                .any(|row| row.evidence.contains(&ContextRule::ImperativeObject))
        );
    }
    let result = analyzer()
        .parse_context(["он", ".", "мой", "чашку"])
        .unwrap();
    assert!(
        result[2]
            .decision
            .support
            .iter()
            .any(|row| row.evidence.contains(&ContextRule::ImperativeObject))
    );
}

#[test]
fn context_does_not_replace_word_cache_or_other_contexts() {
    let morph = analyzer().clone();
    let before = morph.parse("стали").unwrap();
    let first = morph.parse_context(["мы", "стали"]).unwrap();
    let mut second = morph.parse_context(["без", "стали"]).unwrap();
    assert_eq!(first[1].decision.selected.as_ref().unwrap().pos, "VERB");
    assert_eq!(second[1].decision.selected.as_ref().unwrap().pos, "NOUN");
    second[1].candidates.clear();
    second[1].decision.retained.clear();
    assert_eq!(morph.parse("стали").unwrap(), before);
    assert_eq!(morph.parse_context(["мы", "стали"]).unwrap(), first);
}

#[test]
fn context_uses_the_configured_word_ranking() {
    let morph = analyzer().clone().with_probabilities(None);
    let tokens = ["Мы", "будем", "вести", "."];
    let result = morph.parse_context(tokens).unwrap();
    for (token, analysis) in tokens.iter().zip(result) {
        assert_eq!(analysis.candidates, morph.parse(token).unwrap());
    }
}
