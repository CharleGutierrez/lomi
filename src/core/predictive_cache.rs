use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixCacheEntry {
    pub prefix_hash: u64,
    pub prefix_snippet: String,
    pub cached_response: String,
    pub hit_count: u64,
    pub last_accessed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEvalResult {
    pub is_hit: bool,
    pub prefix_hash: u64,
    pub cached_response: Option<String>,
    pub hit_count: u64,
}

pub struct PredictiveCache;

impl PredictiveCache {
    const CACHE_FILE: &'static str = ".lomi_cache/prefix_cache.json";

    /// Computes 64-bit fast hash of system prompt / prefix text
    pub fn compute_prefix_hash(prefix_text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        prefix_text.trim().hash(&mut hasher);
        hasher.finish()
    }

    /// Evaluates whether a prompt prefix has a valid predictive cache hit
    pub fn evaluate_prefix(prefix_text: &str) -> CacheEvalResult {
        let hash = Self::compute_prefix_hash(prefix_text);
        let entries = Self::load_cache();

        if let Some(entry) = entries.get(&hash) {
            let mut updated_entries = entries.clone();
            if let Some(e) = updated_entries.get_mut(&hash) {
                e.hit_count += 1;
                e.last_accessed = chrono::Utc::now().to_rfc3339();
            }

            Self::save_cache(&updated_entries);

            CacheEvalResult {
                is_hit: true,
                prefix_hash: hash,
                cached_response: Some(entry.cached_response.clone()),
                hit_count: entry.hit_count + 1,
            }
        } else {
            CacheEvalResult {
                is_hit: false,
                prefix_hash: hash,
                cached_response: None,
                hit_count: 0,
            }
        }
    }

    /// Inserts a new prefix prompt cache entry into .lomi_cache/prefix_cache.json
    pub fn store_prefix(prefix_text: &str, response: &str) {
        let hash = Self::compute_prefix_hash(prefix_text);
        let mut entries = Self::load_cache();

        let snippet = if prefix_text.len() > 100 {
            format!("{}...", &prefix_text[..100])
        } else {
            prefix_text.to_string()
        };

        let entry = PrefixCacheEntry {
            prefix_hash: hash,
            prefix_snippet: snippet,
            cached_response: response.to_string(),
            hit_count: 1,
            last_accessed: chrono::Utc::now().to_rfc3339(),
        };

        entries.insert(hash, entry);
        Self::save_cache(&entries);
    }

    fn load_cache() -> HashMap<u64, PrefixCacheEntry> {
        if Path::new(Self::CACHE_FILE).exists() {
            if let Ok(content) = fs::read_to_string(Self::CACHE_FILE) {
                if let Ok(map) = serde_json::from_str::<HashMap<u64, PrefixCacheEntry>>(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    fn save_cache(entries: &HashMap<u64, PrefixCacheEntry>) {
        let _ = fs::create_dir_all(".lomi_cache");
        if let Ok(serialized) = serde_json::to_string_pretty(entries) {
            let _ = fs::write(Self::CACHE_FILE, serialized);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_cache_store_and_hit() {
        let prefix = "You are LOMI AI Operating System Assistant v1.0.";
        let response = "LOMI AGI Gateway Ready.";

        PredictiveCache::store_prefix(prefix, response);
        let eval = PredictiveCache::evaluate_prefix(prefix);

        assert!(eval.is_hit);
        assert_eq!(eval.cached_response.unwrap(), response);
        assert!(eval.hit_count >= 1);
    }
}
