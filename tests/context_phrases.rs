use rs_morphy4::{ContextRule, MorphAnalyzer};

struct PhraseCase {
    text: &'static str,
    target: usize,
    lemma: &'static str,
    pos: &'static str,
}

#[test]
fn grammatical_phrases_select_the_expected_reading() {
    let morph = MorphAnalyzer::default();
    let cases = [
        PhraseCase {
            text: "Суши ткань .",
            target: 0,
            lemma: "сушить",
            pos: "VERB",
        },
        PhraseCase {
            text: "Остались две вести .",
            target: 2,
            lemma: "весть",
            pos: "NOUN",
        },
        PhraseCase {
            text: "Она начала печь хлеб .",
            target: 2,
            lemma: "печь",
            pos: "INFN",
        },
        PhraseCase {
            text: "В углу будет печь .",
            target: 3,
            lemma: "печь",
            pos: "NOUN",
        },
        PhraseCase {
            text: "Не суши ткань .",
            target: 1,
            lemma: "сушить",
            pos: "VERB",
        },
        PhraseCase {
            text: "Суши одежду .",
            target: 0,
            lemma: "сушить",
            pos: "VERB",
        },
        PhraseCase {
            text: "Режь хлеб .",
            target: 0,
            lemma: "резать",
            pos: "VERB",
        },
        PhraseCase {
            text: "Две печи .",
            target: 1,
            lemma: "печь",
            pos: "NOUN",
        },
        PhraseCase {
            text: "Четыре стали .",
            target: 1,
            lemma: "сталь",
            pos: "NOUN",
        },
        PhraseCase {
            text: "Она начала мести двор .",
            target: 2,
            lemma: "мести",
            pos: "INFN",
        },
        PhraseCase {
            text: "Она не начала печь хлеб .",
            target: 3,
            lemma: "печь",
            pos: "INFN",
        },
        PhraseCase {
            text: "В комнате будет печь .",
            target: 3,
            lemma: "печь",
            pos: "NOUN",
        },
        PhraseCase {
            text: "В углу будет печь хлеб .",
            target: 3,
            lemma: "печь",
            pos: "INFN",
        },
    ];
    let mut failures = Vec::new();
    for case in cases {
        let result = morph.parse_context(case.text.split_whitespace()).unwrap();
        let selected = result[case.target]
            .decision
            .selected
            .as_ref()
            .map(|r| (r.lemma.as_str(), r.pos.as_str()));
        if selected != Some((case.lemma, case.pos)) {
            failures.push(format!("{}: {:?}", case.text, result[case.target].decision));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn possessive_phrases_do_not_become_imperatives() {
    let morph = MorphAnalyzer::default();
    for text in ["Мой дом .", "Мой хлеб .", "Мой плащ .", "Мой двор ."] {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        assert_eq!(
            result[0].decision.selected.as_ref().map(|r| r.pos.as_str()),
            Some("ADJF"),
            "{text}"
        );
    }
}

#[test]
fn lexical_and_clause_ambiguity_is_not_hidden() {
    let morph = MorphAnalyzer::default();
    for (text, target, lemmas) in [
        (
            "Свежие суши лежали на тарелке .",
            1,
            vec!["суши", "суша", "сушь"],
        ),
        ("Жгут ветки во дворе .", 0, vec!["жечь", "жгут"]),
        ("Жгут провода на столе .", 0, vec!["жечь", "жгут"]),
        ("Я плачу .", 1, vec!["плакать", "платить"]),
        ("Читальные залы .", 1, vec!["зал", "зала"]),
    ] {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        let decision = &result[target].decision;
        assert!(decision.selected.is_none(), "{text}: {decision:?}");
        for lemma in lemmas {
            assert!(
                decision.retained.iter().any(|r| r.lemma == lemma),
                "{text}: {decision:?}"
            );
        }
    }
}

#[test]
fn ambiguous_numeral_does_not_force_a_noun_reading() {
    let morph = MorphAnalyzer::default();
    let result = morph.parse_context(["Три", "пилы", "."]).unwrap();
    assert!(result[0].candidates.iter().any(|p| p.tag.pos == "NUMR"));
    assert!(result[0].candidates.iter().any(|p| p.tag.pos == "VERB"));
    assert!(result[1].decision.selected.is_none());
    assert!(
        result[1]
            .decision
            .retained
            .iter()
            .any(|r| r.lemma == "пила")
    );
}

#[test]
fn numeral_agreement_preserves_case_and_gender() {
    let morph = MorphAnalyzer::default();
    for (numeral, noun, case) in [
        ("две", "вести", "gent"),
        ("двум", "вестям", "datv"),
        ("двумя", "вестями", "ablt"),
        ("двух", "вестях", "loct"),
    ] {
        let result = morph.parse_context([numeral, noun]).unwrap();
        let selected = result[1].decision.selected.as_ref().unwrap();
        assert_eq!(selected.lemma, "весть");
        assert!(
            selected
                .parse_indices
                .iter()
                .all(|index| result[1].candidates[*index].tag.contains(case))
        );
        assert!(
            result[1]
                .decision
                .support
                .iter()
                .any(|r| r.evidence.contains(&ContextRule::NumeralAgreement))
        );
    }
    let result = morph.parse_context(["Две", "пилы"]).unwrap();
    assert_eq!(result[1].decision.selected.as_ref().unwrap().lemma, "пила");
    let result = morph.parse_context(["Два", "вести"]).unwrap();
    assert!(
        !result[1]
            .decision
            .support
            .iter()
            .any(|r| r.evidence.contains(&ContextRule::NumeralAgreement))
    );
}

#[test]
fn existence_rule_requires_a_complete_locative_clause() {
    let morph = MorphAnalyzer::default();
    for (text, target) in [
        ("К углу будет печь .", 3),
        ("В угол будет печь .", 3),
        ("В углу будет , печь .", 4),
        ("Он в углу будет печь .", 4),
        ("В углу будет печь хлеб .", 3),
    ] {
        let result = morph.parse_context(text.split_whitespace()).unwrap();
        assert!(
            !result[target]
                .decision
                .support
                .iter()
                .any(|r| r.evidence.contains(&ContextRule::LocativeExistence)),
            "{text}"
        );
    }
    let result = morph
        .parse_context(["На", "столе", "будет", "пила", "."])
        .unwrap();
    let selected = result[3].decision.selected.as_ref().unwrap();
    assert_eq!(
        (selected.lemma.as_str(), selected.pos.as_str()),
        ("пила", "NOUN")
    );
}
