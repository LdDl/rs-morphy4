# rs-morphy4

[![Crates.io version](https://img.shields.io/crates/v/rs-morphy4.svg)](https://crates.io/crates/rs-morphy4)

[Русская версия](README_RU.md)

[Another](https://www.youtube.com/watch?v=jEI3N9kIyP4) Russian morphological analysis and inflection library (this time implemented in Rust): lemmas, grammatical tags, unknown-word prediction, compound words, lexemes and agreement with integer counts. The library processes individual tokens; tokenization and contextual disambiguation are outside its scope.

## Table of contents

- [Usage](#usage)
- [Behavior](#behavior)
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
};
let morph = MorphAnalyzer::new(dictionary)?;
```

Paths in `include_bytes!` are relative to the Rust file containing this code and are needed only during compilation. A custom dictionary must have the same file structure as data/dicts_ru. Russian dictionaries in pymorphy format 2.4 are supported; anything else will naturally cause a loading error. `DictionaryData` also accepts data from memory instead of embedded files. `MorphAnalyzer::load("path/to/dictionary")` remains available for loading a dictionary from a directory at runtime.

The `tag`, `normal_forms` and `word_is_known` methods provide individual analysis results. The `lexeme`, `normalized`, `inflect` and `make_agree_with_number` methods accept a `Parse` from the same dictionary. If no suitable form exists, inflection returns `None`; invalid dictionary references return typed errors. `DictionaryAnalyzer` provides dictionary-only lookup.

## Behavior

It is simple. Leading and trailing whitespace is removed, and input is lowercased. Empty input produces no analyses. Stable result ordering is determined by whether the weight is below 1, part-of-speech priority, nominative case and feature count. The weight is not a probability (!!!); results are not sorted by descending numeric weight. `parse_unranked` preserves rule order and bypasses the cache.

Analyzer clones share an immutable dictionary and a thread-safe FIFO cache. By default, it holds 1024 normalized tokens. `MorphAnalyzer::from_shared(morph.shared_dictionary(), capacity)` creates an analyzer with a separate cache; capacity 0 disables storage. Results are returned as independent copies. Concurrent cache misses may cause the same token to be computed more than once.

Known limitations/assumptions:

- Handling of the letters `е` and `ё` swaps all occurrences simultaneously under certain conditions; individual combinations of substitutions are not enumerated.
- Unknown and compound words are handled by rules. Synthetic analyses without a paradigm have a lexeme containing one form.
- Agreement with a negative number may produce a different word form than agreement with its positive counterpart: for example, `1 кошка`, but `-1 кошек`. This is a characteristic of the current `make_agree_with_number` algorithm.
- Table structure is checked during loading; DAWG transitions and records are checked when accessed.

## Examples and verification

Examples and tests can be run from the repository root:

```sh
cargo run --example analyze -- стали
cargo run --example inflect
cargo run --example custom_dictionary
cargo test --locked
cargo doc --no-deps
```

The final corpus check covers a set of inputs and ordered analyses. All compared fields matched the pinned reference on this corpus. For those interested, the [test description](tests/README.md) documents coverage and intentional differences.

My configuration for the benchmarks:

```text
Ryzen 7 7800X3D
Linux x86_64
Rust 1.98.0
optimized build
```

These results use dictionary loading through `load(path)`, before embedding was added. Separate measurements for `default()` and `new(dictionary)` have not been taken yet; I do not think they are necessary. The table shows medians of five runs per mode.

| Cache mode | Parse calls/s | Dictionary loading, ms | Process RSS after processing, MiB |
| --- | ---: | ---: | ---: |
| No cache | 401,583 | 8.36 | 15.3 |
| 1024 entries (default) | 361,941 | 8.28 | 17.3 |
| 2504 entries (entire corpus) | 1,754,965 | 7.93 | 19.7 |

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
