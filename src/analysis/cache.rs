use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::analysis::parse::Parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Snapshot of a FIFO cache; capacity limits token count rather than bytes.
pub struct CacheInfo {
    /// Maximum number of normalized tokens; zero disables storage.
    pub capacity: usize,
    /// Number of currently stored tokens.
    pub entries: usize,
    /// Successful lookups since creation or the last clear.
    pub hits: u64,
    /// Unsuccessful lookups, including calls with storage disabled; blank input is excluded.
    pub misses: u64,
}

#[derive(Debug, Default)]
struct State {
    entries: HashMap<String, Vec<Parse>>,
    order: VecDeque<String>,
    hits: u64,
    misses: u64,
}

/// FIFO eviction bounds the number of cached tokens. Values are copied on retrieval.
#[derive(Debug)]
pub struct ParseCache {
    capacity: usize,
    state: Mutex<State>,
}

impl ParseCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Mutex::new(State::default()),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<Parse>> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(value) = state.entries.get(key).cloned() {
            state.hits = state.hits.saturating_add(1);
            Some(value)
        } else {
            state.misses = state.misses.saturating_add(1);
            None
        }
    }

    pub fn insert(&self, key: String, value: Vec<Parse>) {
        if self.capacity == 0 {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.entries.contains_key(&key) {
            return;
        }
        if state.entries.len() == self.capacity
            && let Some(oldest) = state.order.pop_front()
        {
            state.entries.remove(&oldest);
        }
        state.order.push_back(key.clone());
        state.entries.insert(key, value);
    }

    pub fn info(&self) -> CacheInfo {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        CacheInfo {
            capacity: self.capacity,
            entries: state.entries.len(),
            hits: state.hits,
            misses: state.misses,
        }
    }

    pub fn clear(&self) {
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = State::default();
    }
}
