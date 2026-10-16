use rs_morphy4::{ContextRule, MorphAnalyzer};

struct SentenceCase {
    text: &'static str,
    target: usize,
    lemma: &'static str,
}

#[test]
fn future_and_modal_constructions_select_infinitives() {
    let morph = MorphAnalyzer::default();
    let cases = [
        SentenceCase {
            text: "Будут вести переговоры .",
            target: 1,
            lemma: "вести",
        },
        SentenceCase {
            text: "Они смогут мести двор .",
            target: 2,
            lemma: "мести",
        },
        SentenceCase {
            text: "Будут не вести переговоры .",
            target: 2,
            lemma: "вести",
        },
        SentenceCase {
            text: "Не будут вести переговоры .",
            target: 2,
            lemma: "вести",
        },
        SentenceCase {
            text: "Будет печь хлеб .",
            target: 1,
            lemma: "печь",
        },
        SentenceCase {
            text: "Будут мести улицу .",
            target: 1,
            lemma: "мести",
        },
        SentenceCase {
            text: "Будет мыть чашку .",
            target: 1,
            lemma: "мыть",
        },
        SentenceCase {
            text: "Она смогла печь хлеб .",
            target: 2,
            lemma: "печь",
        },
        SentenceCase {
            text: "Мы сможем не мести двор .",
            target: 3,
            lemma: "мести",
        },
        SentenceCase {
            text: "Они не смогут мести двор .",
            target: 3,
            lemma: "мести",
        },
        SentenceCase {
            text: "Она сможет стеречь дом .",
            target: 2,
            lemma: "стеречь",
        },
    ];
    let mut failures = Vec::new();
    for case in cases {
        let result = morph.parse_context(case.text.split_whitespace()).unwrap();
        let decision = &result[case.target].decision;
        let selected = decision
            .selected
            .as_ref()
            .map(|r| (r.lemma.as_str(), r.pos.as_str()));
        if selected != Some((case.lemma, "INFN")) {
            failures.push(format!("{}: {:?}", case.text, decision.retained));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn unsupported_fragments_preserve_noun_and_infinitive() {
    let morph = MorphAnalyzer::default();
    for text in [
        "Будет печь .",
        "Будут вести .",
        "Будет печь мастера .",
        "Смогут мести .",
    ] {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        let decision = &result[1].decision;
        assert!(decision.selected.is_none(), "{text}: {decision:?}");
        for pos in ["NOUN", "INFN"] {
            assert!(
                decision.retained.iter().any(|r| r.pos == pos),
                "{text}: {decision:?}"
            );
        }
    }
}

#[test]
fn future_does_not_link_across_boundaries_or_incompatible_objects() {
    let morph = MorphAnalyzer::default();
    let cases = [
        ("Будут , вести переговоры .", 2),
        ("Будут . Вести переговоры .", 2),
        ("Будут вести , переговоры .", 1),
        ("Будут вести переговоров .", 1),
        ("Будут вести к дому .", 1),
        ("Они смогут , мести двор .", 3),
    ];
    for (text, target) in cases {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        assert!(
            !result[target]
                .decision
                .support
                .iter()
                .any(|row| row.evidence.contains(&ContextRule::GovernedInfinitive)),
            "{text}"
        );
        assert!(
            result[target].decision.selected.is_none(),
            "{text}: {:?}",
            result[target].decision
        );
    }
}

#[test]
fn noun_subjects_and_verb_ambiguity_are_preserved() {
    let morph = MorphAnalyzer::default();
    for text in ["Стояла печь .", "Пришли вести .", "Будет новая печь ."]
    {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        let target = result.len() - 2;
        assert_eq!(
            result[target]
                .decision
                .selected
                .as_ref()
                .map(|r| r.pos.as_str()),
            Some("NOUN"),
            "{text}"
        );
    }
    let result = morph.parse_context(["Я", "плачу", "."]).unwrap();
    assert!(result[1].decision.selected.is_none());
    assert_eq!(result[1].decision.retained.len(), 2);
}

#[test]
fn ambiguous_governor_retains_the_infinitive_without_forcing_selection() {
    let morph = MorphAnalyzer::default();
    let result = morph
        .parse_context(["Смогу", "вести", "переговоры", "."])
        .unwrap();
    assert!(
        result[0]
            .candidates
            .iter()
            .any(|parse| parse.tag.pos == "NOUN")
    );
    assert!(
        result[0]
            .candidates
            .iter()
            .any(|parse| parse.tag.pos == "VERB")
    );
    assert!(result[1].decision.selected.is_none());
    assert!(
        result[1]
            .decision
            .retained
            .iter()
            .any(|reading| reading.lemma == "вести" && reading.pos == "INFN")
    );
}
