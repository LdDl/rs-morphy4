# rs-morphy4

[![Crates.io version](https://img.shields.io/crates/v/rs-morphy4.svg)](https://crates.io/crates/rs-morphy4)

[Русская версия](README_RU.md)

[Another](https://www.youtube.com/watch?v=jEI3N9kIyP4) Russian morphological analysis and inflection library (this time implemented in Rust): lemmas, grammatical tags, unknown-word prediction, compound words, lexemes, agreement with integer counts and contextual analysis selection.

## Table of contents

- [Usage](#usage)
- [Behavior](#behavior)
- [Context analysis](#context-analysis)
- [Examples and verification](#examples-and-verification)
- [DAWG and references](#dawg-and-references)
- [Attribution and licenses](#attribution-and-licenses)

## Usage

It is straightforward: a `MorphAnalyzer` is created with the embedded dictionary, then its analysis and inflection methods are called.

Example:

```rust
use rs_morphy4::MorphAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let morph = MorphAnalyzer::default();
    for parse in morph.parse("кошки")? {
        println!("{}: {}", parse.normal_form, parse.tag);
        if let Some(form) = morph.inflect(&parse, ["plur", "gent"])? {
            println!("{}", form.word);
        }
    }
    Ok(())
}
```

`MorphAnalyzer::default()` uses the ready-made Russian dictionary embedded in the library (literally the one from this repository). No path configuration or separate files are needed at runtime.

`MorphAnalyzer::new(dictionary)` accepts a custom dictionary through the `DictionaryData` structure. Its files can also be embedded in an application using `include_bytes!`:

```rust
use rs_morphy4::{DictionaryData, MorphAnalyzer};

let dictionary = DictionaryData {
    metadata: include_bytes!("my_dictionary/meta.json"),
    grammemes: include_bytes!("my_dictionary/grammemes.json"),
    tags: include_bytes!("my_dictionary/gramtab-opencorpora-int.json"),
    suffixes: include_bytes!("my_dictionary/suffixes.json"),
    paradigms: include_bytes!("my_dictionary/paradigms.array"),
    words: include_bytes!("my_dictionary/words.dawg"),
    prediction_suffixes: None,
};
let morph = MorphAnalyzer::new(dictionary)?;
```

Russian dictionaries in pymorphy format 2.4 use the file structure shown in `data/dicts_ru`. `DictionaryData` accepts bytes from memory or `include_bytes!`; macro paths are relative to the Rust file. `MorphAnalyzer::load("path/to/dictionary")` loads a dictionary directory at runtime.

`prediction_suffixes` accepts suffix prediction tables in paradigm-prefix order. With `None`, fallback rules apply. The embedded dictionary includes the complete set; `load(path)` reads these tables when present.

`tag`, `normal_forms` and `word_is_known` return tags, lemmas and dictionary membership. `lexeme`, `normalized`, `inflect` and `make_agree_with_number` operate on a selected `Parse` from the same dictionary. Inflection returns `None` when no suitable form exists; invalid paradigm references return `InflectionError`. `DictionaryAnalyzer` provides dictionary-only lookup.

## Behavior

It is simple. Leading and trailing whitespace is removed, and input is lowercased. Empty input produces no analyses. `default`, `new` and `load` use the embedded word/tag frequencies to rank results. Equal or missing frequencies use rule weights, then part of speech, nominative case and feature count. `Parse.score` remains a rule weight, not a probability. `parse_unranked` preserves rule order and bypasses the cache.

`morph.with_probabilities(None)` disables statistical ranking; `morph.with_probabilities(Some(Arc::new(model)))` attaches a custom `WordTagProbabilities`.

Analyzer clones share a dictionary and a thread-safe FIFO cache holding 1024 tokens. `MorphAnalyzer::from_shared(morph.shared_dictionary(), capacity)` creates an analyzer with heuristic ranking and a separate cache of the specified size; capacity 0 disables caching.

Known limitations/assumptions:

- Dictionary lookup checks independent `е` -> `ё` substitutions along existing DAWG branches. Explicit `ё` is preserved; prediction is used only when dictionary lookup finds no analyses.
- Unknown words use dictionary roots and suffix models. Prediction can return multiple alternatives. Analyses without a paradigm have a lexeme containing one form.
- Agreement with a negative number may produce a different word form than agreement with its positive counterpart: for example, `1 кошка`, but `-1 кошек`. This is a characteristic of the current `make_agree_with_number` algorithm.
- Word/tag frequencies do not use sentence context. The first analysis can still be unsuitable for a particular sentence.
- Table structure is checked during loading; DAWG transitions and records are checked when accessed.

## Context analysis

`parse_context(tokens)` analyzes a sequence of tokens using neighboring words. Punctuation is supplied as separate tokens.

```rust
let result = morph.parse_context(["Мы", "будем", "вести", "переговоры", "."])?;
let reading = result[2].decision.selected.as_ref().unwrap();
assert_eq!((reading.lemma.as_str(), reading.pos.as_str()), ("вести", "INFN"));
```

The result contains the original `candidates`, the selected lemma and part of speech in `decision.selected`, alternatives in `decision.retained` and applied rules in `decision.support`. Ambiguity leaves `selected` as `None`: for example, "Я плачу" retains both платить and плакать. Rule weights are not probabilities.

Context rules consider agreement and government. These local heuristics can make mistakes and do not always resolve semantic ambiguity.

## Examples and verification

Examples and tests can be run from the repository root:

```sh
cargo run --example analyze -- стали
cargo run --example inflect
cargo run --example custom_dictionary
cargo test --locked
cargo doc --no-deps
```

Test coverage and limitations are described in [tests/README.md](tests/README.md).

The measurements below cover heuristic ranking with suffix prediction. Word/tag frequencies and context analysis are disabled.

My configuration for the benchmarks:

```text
Ryzen 7 7800X3D
Linux x86_64
Rust 1.98.0
optimized build
```

Dictionary loading uses `load(path)`. The table shows medians of five runs per mode; loading through `default()` and `new(dictionary)` is not measured.

| Cache mode | Parse calls/s | Dictionary loading, ms | Process RSS after processing, MiB |
| --- | ---: | ---: | ---: |
| No cache | 326,936 | 9.76 | 22.9 |
| 1024 entries (default) | 300,741 | 9.78 | 22.9 |
| 2504 entries (entire corpus) | 1,715,518 | 9.48 | 23.7 |

The cache was warmed with one corpus pass before measurement. At capacity 1024, the corpus does not fit in the cache: cyclic traversal evicts entries before their next use, so there are no hits. At capacity 2504, all nonempty inputs are found in the cache.

RSS reflects memory used by the entire process, including the dictionary and cache. Loading time was measured without clearing the OS filesystem cache. The [methodology and raw measurements](benchmarks/README.md) contain detailed conditions and all run results.

## DAWG and references

To keep it VERY BRIEF, a DAWG (Directed Acyclic Word Graph) represents a set of strings as a directed acyclic graph. In its dictionary form, it is a deterministic finite automaton: states with identical allowed continuations are merged, allowing compact dictionary storage. Construction of these automata is described in:

- J. Daciuk, S. Mihov, B. W. Watson, R. E. Watson. [Incremental Construction of Minimal Acyclic Finite-State Automata](https://aclanthology.org/J00-1002/). Computational Linguistics, 26(1), 2000, pp. 3-16. [PDF](https://aclanthology.org/J00-1002.pdf).

The practical implementation of this structure in [dawgdic](https://github.com/stil/dawgdic) uses a double-array representation. That is what the [DAWG](https://github.com/pytries/DAWG) library is based on. In `rs-morphy4`, the [dictionary/dawg](src/dictionary/dawg) module reads the prebuilt binary index and extracts records containing paradigm and form identifiers. I have not implemented automaton construction and minimization in the current version of the library. Perhaps I will in the future if there is a need. Still, a Russian dictionary with 2.5 million words and 1.5 million forms takes only about ten MB, so I do not see much need for it.

## Attribution and licenses

Complicated and simple at the same time:

1. The Rust implementation and supporting files are distributed under Apache-2.0.
2. Adapted parts of the dictionary and DAWG (including the encoding) are distributed under MIT.
3. The dictionary is distributed under CC BY-SA 3.0.
