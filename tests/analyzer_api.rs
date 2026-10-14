use std::path::Path;
use std::sync::{Arc, LazyLock};

use rs_morphy4::{DictionaryAnalyzer, MorphAnalyzer};

static DICTIONARY: LazyLock<Arc<DictionaryAnalyzer>> = LazyLock::new(|| {
    Arc::new(
        DictionaryAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru"))
            .unwrap(),
    )
});

fn analyzer(capacity: usize) -> MorphAnalyzer {
    MorphAnalyzer::from_shared(Arc::clone(&DICTIONARY), capacity)
}

#[test]
fn normalized_cache_keys_and_owned_results_are_independent() {
    let analyzer = analyzer(2);
    let expected = analyzer.parse("КОШКИ").unwrap();
    let mut changed = analyzer.parse("  кошки\t").unwrap();
    changed[0].word.clear();
    changed[0].tag.grammemes.clear();
    changed.clear();
    assert_eq!(analyzer.parse("кошки").unwrap(), expected);
    let info = analyzer.cache_info();
    assert_eq!(
        (info.capacity, info.entries, info.hits, info.misses),
        (2, 1, 2, 1)
    );
}

#[test]
fn fifo_eviction_preserves_the_configured_bound() {
    let analyzer = analyzer(2);
    for word in ["дом", "кот", "дом", "лес", "кот"] {
        analyzer.parse(word).unwrap();
    }
    let info = analyzer.cache_info();
    assert_eq!((info.entries, info.hits, info.misses), (2, 2, 3));
    analyzer.parse("дом").unwrap();
    assert_eq!(analyzer.cache_info().misses, 4);
    assert_eq!(analyzer.cache_info().entries, 2);
}

#[test]
fn clones_share_cache_and_dictionary_but_new_analyzers_can_have_separate_caches() {
    let analyzer = analyzer(2);
    let cloned = analyzer.clone();
    let separate = MorphAnalyzer::from_shared(analyzer.shared_dictionary(), 1);
    assert!(Arc::ptr_eq(
        &analyzer.shared_dictionary(),
        &cloned.shared_dictionary()
    ));
    assert!(Arc::ptr_eq(
        &analyzer.shared_dictionary(),
        &separate.shared_dictionary()
    ));
    analyzer.parse("дом").unwrap();
    cloned.parse("дом").unwrap();
    assert_eq!(analyzer.cache_info().hits, 1);
    assert_eq!(separate.cache_info().entries, 0);
    separate.parse("дом").unwrap();
    cloned.clear_cache();
    assert_eq!(analyzer.cache_info().entries, 0);
    assert_eq!(analyzer.cache_info().hits, 0);
    assert_eq!(analyzer.cache_info().misses, 0);
    assert_eq!(separate.cache_info().entries, 1);
}

#[test]
fn zero_capacity_disables_storage_and_unranked_api_bypasses_cache() {
    let analyzer = analyzer(0);
    let first = analyzer.parse("стали").unwrap();
    assert_eq!(analyzer.parse("стали").unwrap(), first);
    assert_eq!(
        (
            analyzer.cache_info().entries,
            analyzer.cache_info().hits,
            analyzer.cache_info().misses
        ),
        (0, 0, 2)
    );
    let before = analyzer.cache_info();
    analyzer.parse_unranked("стали").unwrap();
    assert!(analyzer.parse(" \t\n").unwrap().is_empty());
    assert_eq!(analyzer.cache_info(), before);
}

#[test]
fn api_views_follow_ranking_and_dictionary_membership() {
    let analyzer = analyzer(4);
    let parsed = analyzer.parse("стали").unwrap();
    assert_eq!(parsed[0].tag.pos, "NOUN");
    assert_eq!(analyzer.parse_unranked("стали").unwrap()[0].tag.pos, "VERB");
    assert_eq!(
        analyzer.tag("стали").unwrap(),
        parsed.iter().map(|p| p.tag.clone()).collect::<Vec<_>>()
    );
    assert_eq!(analyzer.normal_forms("стали").unwrap(), ["сталь", "стать"]);
    assert!(analyzer.word_is_known(" ЁЛКА ").unwrap());
    assert!(analyzer.word_is_known("елка").unwrap());
    assert!(!analyzer.word_is_known("зафырзуют").unwrap());
    assert!(!analyzer.word_is_known("123").unwrap());
    let unranked = analyzer.parse_unranked("ѯой").unwrap();
    let ranked = analyzer.parse("ѯой").unwrap();
    assert_eq!(unranked[0].tag.pos, "ADJF");
    assert_eq!(ranked[0].tag.pos, "NOUN");
    assert_eq!(ranked[0].score, 0.4);
}

#[test]
fn concurrent_shared_analyzers_preserve_results_and_cache_bounds() {
    let analyzer = analyzer(2);
    let words = ["стали", "кошки", "елка", "pdf-документов"];
    let uncached = MorphAnalyzer::from_shared(analyzer.shared_dictionary(), 0);
    let expected: Vec<_> = words.iter().map(|w| uncached.parse(w).unwrap()).collect();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let shared = analyzer.clone();
            let expected = &expected;
            scope.spawn(move || {
                for iteration in 0..50 {
                    let index = iteration % words.len();
                    assert_eq!(shared.parse(words[index]).unwrap(), expected[index]);
                }
            });
        }
    });
    let info = analyzer.cache_info();
    assert!(info.entries <= 2);
    assert_eq!(info.hits + info.misses, 200);
}
