# Russian dictionary

[Русская версия](README_RU.md).

`dicts_ru/` contains the compiled Russian morphological dictionary from [mawo-pymorphy3](https://github.com/mawo-ru/mawo-pymorphy3/tree/f60ff89ced0aab256470061e1e566fab317b6170/mawo_pymorphy3/dicts_ru), commit `f60ff89ced0aab256470061e1e566fab317b6170`. All 10 files are preserved without changes.

The dictionary originates from [OpenCorpora](https://opencorpora.org/), maintained by the OpenCorpora contributors. The distribution source is the MAWO Team's mawo-pymorphy3 project. According to `dicts_ru/meta.json`, the source revision is `417260`, the compiled format is `2.4`, and the compilation timestamp is `2025-09-15T06:52:07.147219`.

The bundle contains word and suffix-prediction DAWGs, paradigms, suffixes, grammatical tables and metadata. OpenCorpora XML is an input for dictionary compilation; it is not required to read these compiled files and is not included here.

In the future, I may implement my own dictionary compiler to process OpenCorpora XML directly, without using precompiled dictionaries from mawo-pymorphy3. For now, everything remains as it is.

The dictionary data is distributed under [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).

Attribution and license information accompany redistribution; adaptations of the data remain subject to its ShareAlike terms. The license text is in [LICENSE-CC-BY-SA-3.0.txt](LICENSE-CC-BY-SA-3.0.txt), obtained from the [SPDX license list](https://github.com/spdx/license-list-data/blob/main/text/CC-BY-SA-3.0.txt).

The Rust code's Apache-2.0 and MIT notices do not replace the dictionary's license.

SHA-256 checksums are recorded in `SHA256SUMS`. Verification from this directory:

```sh
sha256sum -c SHA256SUMS
```
