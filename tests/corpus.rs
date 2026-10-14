use std::collections::BTreeSet;
use std::path::Path;

use rs_morphy4::{MorphAnalyzer, Parse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    input: String,
    expected: Vec<Analysis>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Analysis {
    word: String,
    normal_form: String,
    pos: String,
    grammemes: BTreeSet<String>,
    score: f64,
}

impl From<Parse> for Analysis {
    fn from(parse: Parse) -> Self {
        Self {
            word: parse.word,
            normal_form: parse.normal_form,
            pos: parse.tag.pos,
            grammemes: parse.tag.grammemes,
            score: parse.score,
        }
    }
}

#[test]
fn ranked_corpus_matches_reference() {
    let corpus: Corpus = serde_json::from_str(include_str!("fixtures/corpus.json")).unwrap();
    let inputs: Vec<String> = serde_json::from_str(include_str!("../benchmarks/corpus.json")).unwrap();
    assert_eq!(corpus.cases.iter().map(|c| &c.input).collect::<Vec<_>>(), inputs.iter().collect::<Vec<_>>());
    let analyzer = MorphAnalyzer::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dicts_ru")).unwrap();
    let mut mismatches = Vec::new();
    for case in corpus.cases {
        let actual: Vec<Analysis> = analyzer.parse(&case.input).unwrap().into_iter().map(Into::into).collect();
        if actual != case.expected {
            mismatches.push(format!("{:?}\nactual: {:?}\nexpected: {:?}", case.input, actual, case.expected));
        }
    }
    assert!(mismatches.is_empty(), "{} mismatches:\n{}", mismatches.len(), mismatches.join("\n"));
}
