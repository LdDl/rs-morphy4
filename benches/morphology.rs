use std::hint::black_box;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use rs_morphy4::{DictionaryAnalyzer, MorphAnalyzer};
use serde_json::json;

fn memory_kib(field: &str) -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name != field { return None; }
        value.split_whitespace().next()?.parse().ok()
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).filter(|s| s != "--bench");
    let directory = args.next().unwrap_or_else(|| "data/dicts_ru".into());
    let corpus = args.next().unwrap_or_else(|| "benchmarks/corpus.json".into());
    let capacity: usize = args.next().unwrap_or_else(|| "0".into()).parse()?;
    let rounds: NonZeroUsize = args.next().unwrap_or_else(|| "40".into()).parse()?;
    let tokens: Vec<String> = serde_json::from_slice(&std::fs::read(corpus)?)?;
    let rss_before = memory_kib("VmRSS");
    let started = Instant::now();
    let analyzer = MorphAnalyzer::from_shared(Arc::new(DictionaryAnalyzer::load(directory)?), capacity);
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let rss_loaded = memory_kib("VmRSS");
    for token in &tokens { black_box(analyzer.parse(black_box(token))?); }
    let rss_warm = memory_kib("VmRSS");
    let cache_before = analyzer.cache_info();
    let started = Instant::now();
    let mut analyses = 0u64;
    for _ in 0..rounds.get() {
        for token in &tokens {
            analyses += black_box(analyzer.parse(black_box(token))?).len() as u64;
        }
    }
    let elapsed = started.elapsed().as_secs_f64();
    let calls = tokens.len() as u128 * rounds.get() as u128;
    let cache_after = analyzer.cache_info();
    println!("{}", json!({
        "inputs": tokens.len(), "rounds": rounds.get(), "calls": calls,
        "analyses": analyses, "cache_capacity": capacity, "cache_entries": cache_after.entries,
        "cache_hits": cache_after.hits - cache_before.hits,
        "cache_misses": cache_after.misses - cache_before.misses,
        "load_ms": load_ms, "elapsed_seconds": elapsed, "calls_per_second": calls as f64 / elapsed,
        "rss_before_load_kib": rss_before, "rss_after_load_kib": rss_loaded,
        "rss_after_warmup_kib": rss_warm, "rss_final_kib": memory_kib("VmRSS"),
        "peak_rss_kib": memory_kib("VmHWM")
    }));
    Ok(())
}
