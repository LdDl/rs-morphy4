# Validation

The initial reference is [mawo-pymorphy3](https://github.com/mawo-ru/mawo-pymorphy3/tree/f60ff89ced0aab256470061e1e566fab317b6170), commit `f60ff89ced0aab256470061e1e566fab317b6170`, with the same `data/dicts_ru` files. Tests use the checked-in JSON fixtures.

| Check | Coverage |
| --- | --- |
| Dictionary fixture | 673 cases |
| Paradigm validation | 3472 paradigms, 142215 forms |
| Analysis rules | 1731 cases in rule order |
| Heuristic ranking | 1720 distinct inputs |
| Word forms | 66 lexemes and normalized forms, 3168 inflection/agreement queries |
| Heuristic corpus | 2504 inputs, 5934 ordered analyses |

The final corpus combines tokens from 30 synthetic Russian news paragraphs with dictionary word forms and targeted cases such as unknown words, numbers, Latin text, compounds and empty input. Its full input list is [benchmarks/corpus.json](../benchmarks/corpus.json), with expected output in [fixtures/corpus.json](fixtures/corpus.json).

The heuristic comparison checks `word`, `normal_form`, part of speech, the complete feature set, score and result order. Spelling expectations include independent dictionary `е` -> `ё` substitutions and preserve explicit `ё`; they were checked against a separate DAWG lookup implementation. Guesses obtained by swapping all letters are excluded. Dictionary provenance fields are not part of that comparison. The corpus has no human-annotated contextual interpretation: agreement with the reference is not a linguistic accuracy score or proof for all dictionary entries.

Suffix prediction checks productive paradigm counts, retained alternatives, inflection, optional tables, corrupt payloads and references. Fourteen proposed lemma/POS cases cover unfamiliar nouns, verbs and adjectives. Normalization uses the selected paradigm rather than reanalyzing its lemma. Fixture scores are decoded with exact floating-point round trips.

Spelling regressions cover missing dots at individual and multiple positions, long words, preservation of explicit `ё`, dictionary precedence over prediction, stem preservation during inflection, malformed payloads and missing DAWG branches. Four lexeme sources consisting of discarded spelling guesses were removed; dictionary lexeme checks remain.

Productive-prefix checks cover dictionary-backed nominal/adjectival roots, dictionary precedence and inflection with the prefix retained. The invariant пол- is checked separately. Corpus expectations reflect these rules; only the affected records were updated.

Additional tests cover malformed dictionary data, typed errors, cache eviction and disabling, shared dictionaries, independent returned values, concurrent calls, repeated inflection, embedded dictionaries and validation of custom dictionary data. The complete suite contains 122 integration tests and two rustdoc examples.

Statistical ranking uses [fixtures/probabilities.json](fixtures/probabilities.json): 5914 word/tag queries with integer frequencies and 2504 expected first lemmas. Frequency data comes from the pinned model described in [data/README.md](../data/README.md). Checks cover missing entries, equivalent tag spellings, unchanged candidate contents and rule weights, disabling/replacing the model, cache isolation, malformed integers and typed model errors. The heuristic reference remains checked with statistics disabled. These checks validate the implementation, not contextual accuracy.

Context checks use [fixtures/context.json](fixtures/context.json): 84 manually selected contexts with frozen decisions, retained lemma/POS pairs, candidate indices and rule evidence. These are behavior regressions, not an independent linguistic accuracy benchmark. Future and modal checks cover omitted subjects, accusative objects, negation, competing noun readings and смочь. Phrase checks cover numeral case/gender agreement, imperatives versus possessives, ambiguous governors with explicit subjects, and locative clauses. "Смогу вести переговоры", "Свежие суши лежали на тарелке" and "Жгут ветки во дворе" remain unresolved. Additional checks cover empty input, punctuation boundaries, negation, modal and imperative constructions, ambiguous prepositions, case government, multiple analyses per reading, unchanged word results, cache isolation and disabled statistical ranking.

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
```
