use std::collections::{BTreeSet, HashSet};

use rs_morphy4::Tag;

#[test]
fn default_is_unknown_without_features() {
    let tag = Tag::default();
    assert_eq!(tag.pos, "UNKN");
    assert!(tag.grammemes.is_empty());
    assert!(tag.contains("UNKN"));
    assert_eq!(tag.to_string(), "UNKN");
}

#[test]
fn constructor_preserves_features_and_removes_duplicates() {
    let tag = Tag::new("NOUN", ["sing", "loc2", "Custom", "sing"]);
    assert_eq!(tag.pos, "NOUN");
    assert_eq!(
        tag.grammemes,
        BTreeSet::from(["Custom".to_owned(), "loc2".to_owned(), "sing".to_owned()])
    );
    assert!(!tag.contains("loct"));
}

#[test]
fn membership_includes_pos_and_is_case_sensitive() {
    let tag = Tag::new("NOUN", ["sing", "nomn"]);
    assert!(tag.contains("NOUN"));
    assert!(tag.contains("sing"));
    assert!(!tag.contains("noun"));
    assert!(!tag.contains("Sing"));
    assert!(!tag.contains("plur"));
    assert!(!tag.contains(""));
}

#[test]
fn display_sorts_features_and_omits_separator_for_empty_set() {
    let tag = Tag::new("NOUN", ["sing", "nomn", "masc", "nomn"]);
    assert_eq!(tag.to_string(), "NOUN masc,nomn,sing");
    assert_eq!(Tag::new("VERB", Vec::<String>::new()).to_string(), "VERB");
}

#[test]
fn equality_and_hash_ignore_feature_order_and_duplicates() {
    let first = Tag::new("NOUN", ["sing", "nomn"]);
    let second = Tag::new("NOUN", ["nomn", "sing", "sing"]);
    assert_eq!(first, second);
    assert_eq!(HashSet::from([first.clone(), second]).len(), 1);
    assert_ne!(first, Tag::new("ADJF", ["nomn", "sing"]));
    assert_ne!(first, Tag::new("NOUN", ["nomn", "plur"]));
}

#[test]
fn each_rare_case_has_a_common_form() {
    for (rare, common) in [
        ("gen1", "gent"),
        ("gen2", "gent"),
        ("acc1", "accs"),
        ("acc2", "accs"),
        ("loc1", "loct"),
        ("loc2", "loct"),
        ("voct", "nomn"),
    ] {
        let input = BTreeSet::from([rare.to_owned()]);
        assert_eq!(
            Tag::fix_rare_cases(&input),
            BTreeSet::from([common.to_owned()]),
            "{rare}"
        );
    }
}

#[test]
fn normalization_preserves_input_and_unrecognized_features() {
    let input = Tag::new("NOUN", ["gen1", "gen2", "gent", "sing", "Gen1", "Custom"]);
    let normalized = Tag::fix_rare_cases(&input.grammemes);
    let expected = Tag::new("NOUN", ["gent", "sing", "Gen1", "Custom"]);
    assert_eq!(normalized, expected.grammemes);
    assert_eq!(Tag::fix_rare_cases(&normalized), normalized);
    assert!(input.contains("gen1"));
    assert!(input.contains("gen2"));
}

#[test]
fn normalization_preserves_an_empty_set() {
    assert!(Tag::fix_rare_cases(&BTreeSet::new()).is_empty());
}

fn properties(tag: &Tag) -> [Option<&str>; 5] {
    [
        tag.case(),
        tag.number(),
        tag.gender(),
        tag.aspect(),
        tag.tense(),
    ]
}

#[test]
fn each_feature_belongs_to_its_category() {
    for (category, values) in [
        &["nomn", "gent", "datv", "accs", "ablt", "loct", "voct"][..],
        &["sing", "plur"],
        &["masc", "femn", "neut"],
        &["perf", "impf"],
        &["past", "pres", "futr"],
    ]
    .iter()
    .enumerate()
    {
        for value in *values {
            let tag = Tag::new("UNKN", [*value, "Custom"]);
            let mut expected = [None; 5];
            expected[category] = Some(*value);
            assert_eq!(properties(&tag), expected, "{value}");
        }
    }
}

#[test]
fn grammatical_properties_are_independent_of_pos() {
    let tag = Tag::new("VERB", ["sing", "masc", "perf", "past", "tran"]);
    assert_eq!(
        properties(&tag),
        [None, Some("sing"), Some("masc"), Some("perf"), Some("past")]
    );
    let pos_only = Tag::new("sing", Vec::<String>::new());
    assert!(pos_only.contains("sing"));
    assert_eq!(properties(&pos_only), [None; 5]);
}

#[test]
fn absent_and_unrecognized_properties_are_none() {
    assert_eq!(properties(&Tag::default()), [None; 5]);
    let tag = Tag::new("NOUN", ["Nomn", "Sing", "MASC", "Perf", "Past", "Custom"]);
    assert_eq!(properties(&tag), [None; 5]);
}

#[test]
fn rare_cases_require_explicit_normalization() {
    for (rare, common) in [
        ("gen1", "gent"),
        ("gen2", "gent"),
        ("acc1", "accs"),
        ("acc2", "accs"),
        ("loc1", "loct"),
        ("loc2", "loct"),
        ("voct", "nomn"),
    ] {
        let tag = Tag::new("NOUN", [rare]);
        assert_eq!(tag.case(), (rare == "voct").then_some("voct"));
        let normalized = Tag::new("NOUN", Tag::fix_rare_cases(&tag.grammemes));
        assert_eq!(normalized.case(), Some(common));
        assert!(tag.contains(rare));
    }
}

#[test]
fn conflicting_properties_use_lexicographic_order() {
    let values = [
        "nomn", "gent", "sing", "plur", "masc", "femn", "perf", "impf", "pres", "past",
    ];
    let forward = Tag::new("NOUN", values);
    let reversed = Tag::new("NOUN", values.into_iter().rev());
    let expected = [
        Some("gent"),
        Some("plur"),
        Some("femn"),
        Some("impf"),
        Some("past"),
    ];
    assert_eq!(properties(&forward), expected);
    assert_eq!(properties(&reversed), expected);
}
