# Measurements

These samples measure heuristic ranking with suffix prediction. Word/tag frequencies and context analysis are disabled.

My configuration for the benchmarks:

```text
Ryzen 7 7800X3D
Linux x86_64
Rust 1.98.0
optimized build
```

The machine has 32GiB RAM (well, approximately free 30.5). The benchmark uses one thread and Cargo's default optimized bench profile, no fancy stuff here.

CPU affinity and frequency are not fixed. Other activity on my machine is not isolated.

Each mode runs in a fresh process, loads the same bundled dictionary, traverses all 2504 inputs once for warmup, then measures 40 complete traversals: 100160 calls and 237360 returned analyses. Five samples per mode are interleaved in capacity order 0, 1024, 2504. Timed work includes result allocation and disposal; dictionary loading, corpus decoding and warmup are excluded.

| Cache capacity | Median calls/s | Min to max calls/s | Median load, ms | Median final RSS, MiB | Median peak RSS, MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 326936 | 302401 to 330100 | 9.76 | 22.9 | 22.9 |
| 1024 | 300741 | 281725 to 305740 | 9.78 | 22.9 | 22.9 |
| 2504 | 1715518 | 1680014 to 1744576 | 9.48 | 23.7 | 23.7 |

The corpus has 2501 distinct nonempty normalized keys and three blank inputs. At capacity 1024, cyclic traversal evicts entries before their next use: all 100040 nonblank measured calls miss the cache. Capacity 2504 retains all keys after warmup: all nonblank measured calls hit. Blank inputs bypass the cache. I hope this workload demonstrates both eviction overhead and warmed-cache throughput, not an assumed production hit rate.

Load timings use fresh processes with an OS filesystem cache that was not cleared at all. So therefore they do not represent cold-storage latency. Memory data comes from Linux `/proc/self/status`: `VmRSS` snapshots and the process high-water mark `VmHWM`. RSS includes the executable, corpus, dictionary, temporary allocations and result cache; it is not an isolated heap-size measurement. Peak RSS includes dictionary loading and warmup. So, you should know that non-Linux systems report unavailable memory fields as null.

[results.json](results.json) contains all 15 samples, environment identifiers and SHA-256 hashes of the source files, corpus and dictionary checksum manifest. The dictionary manifest is [data/SHA256SUMS](../data/SHA256SUMS). The source is [benches/morphology.rs](../benches/morphology.rs).

Equivalent runs from the repository root:

```sh
for sample in 1 2 3 4 5; do
    for capacity in 0 1024 2504; do
        cargo bench --locked --bench morphology -- data/dicts_ru benchmarks/corpus.json "$capacity" 40
    done
done
```

Arguments are dictionary directory, corpus JSON, cache capacity and nonzero round count. Each invocation prints one JSON result. The raw measurements used the built bench executable directly. Compilation was outside the measured interval for sure.
