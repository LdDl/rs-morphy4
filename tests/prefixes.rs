use rs_morphy4::MorphAnalyzer;

#[test]
fn productive_parts_preserve_dictionary_roots() {
    let morph = MorphAnalyzer::default();
    for (word, lemma) in [
        ("нейросервис", "нейросервис"),
        ("нейросервисы", "нейросервис"),
        ("нейросервисами", "нейросервис"),
        ("микросервисами", "микросервис"),
        ("макросервисами", "макросервис"),
        ("мегасервисами", "мегасервис"),
        ("антисервисами", "антисервис"),
        ("суперсервисами", "суперсервис"),
        ("ультрасервисами", "ультрасервис"),
        ("псевдосервисами", "псевдосервис"),
    ] {
        let parses = morph.parse(word).unwrap();
        assert_eq!(parses[0].normal_form, lemma, "{word}");
        assert_eq!(parses[0].tag.pos, "NOUN", "{word}");
    }
}

#[test]
fn prefixed_paradigms_keep_the_prefix_during_inflection() {
    let morph = MorphAnalyzer::default();
    let parse = morph.parse("нейросервисами").unwrap().remove(0);
    assert_eq!(parse.score, 0.75);
    assert_eq!(morph.normalized(&parse).unwrap().normal_form, "нейросервис");
    let form = morph.inflect(&parse, ["sing", "datv"]).unwrap().unwrap();
    assert_eq!(form.word, "нейросервису");
    let forms = morph.lexeme(&parse).unwrap();
    assert!(
        forms
            .iter()
            .all(|form| form.word.starts_with("нейро") && form.normal_form == "нейросервис")
    );
}

#[test]
fn known_words_keep_dictionary_analyses_before_prefix_prediction() {
    let morph = MorphAnalyzer::default();
    for word in ["антиквариат", "микроскоп", "нейрон", "мегаполис"]
    {
        let known = morph.dictionary().parse_with_yo(word).unwrap();
        assert!(!known.is_empty(), "{word}");
        let actual = morph.parse(word).unwrap();
        assert_eq!(actual.len(), known.len(), "{word}");
        assert!(actual.iter().all(|parse| known.contains(parse)), "{word}");
    }
}

#[test]
fn half_prefix_does_not_become_an_adjective() {
    let morph = MorphAnalyzer::default();
    for (word, lemma) in [
        ("пол-яблока", "пол-яблоко"),
        ("пол-лимона", "пол-лимон"),
        ("пол-апельсина", "пол-апельсин"),
    ] {
        let parse = morph.parse(word).unwrap().remove(0);
        assert_eq!(parse.normal_form, lemma, "{word}");
        assert_eq!(parse.tag.pos, "NOUN");
    }
    assert_eq!(morph.parse("пол").unwrap()[0].normal_form, "пол");
}
