# Validation

The reference is [mawo-pymorphy3](https://github.com/mawo-ru/mawo-pymorphy3/tree/f60ff89ced0aab256470061e1e566fab317b6170), commit `f60ff89ced0aab256470061e1e566fab317b6170`, with the same `data/dicts_ru` files. Tests use the checked-in JSON fixtures.

| Check | Coverage |
| --- | --- |
| Dictionary fixture | 673 cases |
| Paradigm validation | 3472 paradigms, 142215 forms |
| Analysis rules | 1731 cases in rule order |
| Ranking | 1720 distinct inputs |
| Word forms | 70 lexemes and normalized forms, 3360 inflection/agreement queries |
| Final corpus | 2504 inputs, 5948 ordered analyses |

The final corpus combines tokens from 30 synthetic Russian news paragraphs with dictionary word forms and targeted cases such as unknown words, numbers, Latin text, compounds and empty input. Its full input list is [benchmarks/corpus.json](../benchmarks/corpus.json), with expected output in [fixtures/corpus.json](fixtures/corpus.json).

The final comparison checks `word`, `normal_form`, part of speech, the complete feature set, score and result order. There were no differences in these fields. Dictionary provenance fields are not part of that comparison. The corpus has no human-annotated contextual interpretation: agreement with the reference is not a linguistic accuracy score or proof for all dictionary entries.

An intentional difference preserves paradigm/stem references after aspect correction of predicted verbs. The reference loses these fields, preventing reconstruction of their lexemes. A separate Rust regression checks preserved provenance; such predictions are excluded from the reference lexeme comparison.

Additional tests cover malformed dictionary data, typed errors, cache eviction and disabling, shared dictionaries, independent returned values, concurrent calls, repeated inflection, embedded dictionaries and validation of custom dictionary data. The complete suite contains 73 integration tests and one rustdoc example.

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
```
