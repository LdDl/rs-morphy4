# Russian dictionary

[Русская версия](README_RU.md).

`dicts_ru/` contains the compiled Russian morphological dictionary from [mawo-pymorphy3](https://github.com/mawo-ru/mawo-pymorphy3/tree/f60ff89ced0aab256470061e1e566fab317b6170/mawo_pymorphy3/dicts_ru), commit `f60ff89ced0aab256470061e1e566fab317b6170`. All 10 files are preserved without changes.

The dictionary originates from [OpenCorpora](https://opencorpora.org/), maintained by the OpenCorpora contributors. The distribution source is the MAWO Team's mawo-pymorphy3 project. According to `dicts_ru/meta.json`, the source revision is `417260` and the compiled format is `2.4`.

The bundle contains word and suffix-prediction DAWGs, paradigms, suffixes, grammatical tables and metadata. OpenCorpora XML is an input for dictionary compilation; it is not required to read these compiled files and is not included here.

In the future, I may implement my own dictionary compiler to process OpenCorpora XML directly, without using precompiled dictionaries from mawo-pymorphy3. For now, everything remains as it is.

The dictionary data is distributed under [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).

Attribution and license information accompany redistribution; adaptations of the data remain subject to its ShareAlike terms. The license text is in [LICENSE-CC-BY-SA-3.0.txt](LICENSE-CC-BY-SA-3.0.txt), obtained from the [SPDX license list](https://github.com/spdx/license-list-data/blob/main/text/CC-BY-SA-3.0.txt).

The Rust code's Apache-2.0 and MIT notices do not replace the dictionary's license.

SHA-256 checksums are recorded in `SHA256SUMS`. Verification from this directory:

```sh
sha256sum -c SHA256SUMS
```

## Word/tag frequencies

`probabilities_ru/` contains `p_t_given_w.intdawg`, `gramtab-opencorpora-int.json` and `meta.json` copied unchanged from [pymorphy3-dicts-ru 2.4.417150.4580142](https://pypi.org/project/pymorphy3-dicts-ru/2.4.417150.4580142/). Distribution maintainer: Danylo Halaiko and contributors. The data originates from OpenCorpora contributors and is distributed under CC BY-SA 3.0. Dictionary revision: 417150; annotated corpus revision: 4580142. This is a separate source from `dicts_ru`, whose revision is 417260.

The integer DAWG stores probabilities in millionths for `word:tag` keys. Tags are matched by part of speech and the complete set of grammatical features. Equivalent tag spellings use the maximum stored value. Absent entries have frequency zero. The model and tag table are embedded; runtime downloads and files are not required. These frequencies do not encode sentence context.
