use std::collections::HashSet;
use std::sync::LazyLock;

use rs_morphy4::dictionary::dawg::{RecordDawg, WordRecord};
use rs_morphy4::{DictionaryAnalyzer, MorphAnalyzer};

use crate::fixture;

static ANALYZER: LazyLock<MorphAnalyzer> = LazyLock::new(MorphAnalyzer::default);
static DICTIONARY: LazyLock<DictionaryAnalyzer> =
    LazyLock::new(|| DictionaryAnalyzer::embedded().unwrap());

#[test]
fn missing_dots_resolve_dictionary_words_before_prediction() {
    for (input, lemma) in [
        ("ведет", "вести"),
        ("ведёт", "вести"),
        ("ребенка", "ребёнок"),
        ("ребёнка", "ребёнок"),
        ("берет", "брать"),
        ("берёт", "брать"),
        ("  РЕБЕНКА  ", "ребёнок"),
    ] {
        let parses = ANALYZER.parse(input).unwrap();
        assert_eq!(parses[0].normal_form, lemma, "{input}");
        assert!(
            parses
                .iter()
                .all(|p| p.paradigm_id.is_some() && p.score == 1.0),
            "{input}"
        );
        assert!(
            parses.iter().all(|p| p.word == input.trim().to_lowercase()),
            "{input}"
        );
        let keys: HashSet<_> = parses.iter().map(|p| (&p.normal_form, &p.tag)).collect();
        assert_eq!(keys.len(), parses.len(), "{input}");
        assert_eq!(ANALYZER.parse(input).unwrap(), parses);
    }
}

#[test]
fn explicit_yo_does_not_introduce_other_dictionary_spellings() {
    let explicit = DICTIONARY.parse("всё").unwrap();
    assert_eq!(explicit, DICTIONARY.parse_exact("всё").unwrap());
    let ambiguous = DICTIONARY.parse("все").unwrap();
    assert!(ambiguous.iter().any(|p| p.normal_form == "весь"));
    assert!(ambiguous.iter().any(|p| p.normal_form == "всё"));
    assert!(DICTIONARY.parse_exact("ребенка").unwrap().is_empty());
    assert!(!DICTIONARY.parse("семенов").unwrap().is_empty());
}

#[test]
fn restored_spelling_retains_stem_for_inflection() {
    let parse = ANALYZER.parse("ребенка").unwrap().remove(0);
    let lexeme = ANALYZER.lexeme(&parse).unwrap();
    assert!(lexeme.iter().any(|p| p.word == "ребёнок"));
    assert!(lexeme.iter().any(|p| p.word == "ребёнка"));
    assert!(lexeme.iter().all(|p| !p.word.starts_with("рёб")));
}

#[test]
fn lookup_handles_independent_positions_and_long_spellings() {
    for (stored, input) in [
        ("ведёт", "ведет"),
        ("берёзёнка", "березенка"),
        ("берёзёнка", "берёзенка"),
        ("переплетённый", "переплетенный"),
        ("ёж", "еж"),
        ("漢ё字", "漢е字"),
    ] {
        let bytes = fixture::DawgFixture::new(stored, b"AQIDBA==").bytes();
        let dawg = RecordDawg::<WordRecord>::from_bytes(&bytes, "test").unwrap();
        let matches = dawg.get_with_yo(input).unwrap();
        assert_eq!(matches.len(), 1, "{input}");
        assert_eq!(matches[0].word, stored);
        assert_eq!(matches[0].records, dawg.get(stored).unwrap());
    }
}

#[test]
fn missing_branches_and_reserved_bytes_have_no_matches() {
    let bytes = fixture::DawgFixture::new("все", b"AQIDBA==").bytes();
    let dawg = RecordDawg::<WordRecord>::from_bytes(&bytes, "test").unwrap();
    for input in ["всё", "", "все\0", "\u{1}", &"е".repeat(100_000)] {
        assert!(dawg.get_with_yo(input).unwrap().is_empty());
    }
}

#[test]
fn invalid_payload_in_a_substitution_branch_returns_error() {
    let bytes = fixture::DawgFixture::new("ёлка", b"!!!!").bytes();
    let dawg = RecordDawg::<WordRecord>::from_bytes(&bytes, "test").unwrap();
    assert!(dawg.get_with_yo("елка").is_err());
}
