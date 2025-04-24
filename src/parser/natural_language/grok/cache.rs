//! Cache module for Grok parser
//!
//! This module provides caching functionality for Grok API requests
//! to avoid redundant API calls and improve performance.

use lru::LruCache;
use once_cell::sync::Lazy;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Cache for storing parsed natural language commands to avoid repeated API calls
///
/// Using a thread-safe LRU cache with a maximum size of 100 entries.
/// The cache is keyed by the sanitized input string and stores the response command.
pub static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

/// Get a cached response if available
///
/// # Arguments
///
/// * `input` - The sanitized input string used as the cache key
///
/// # Returns
///
/// An Option containing the cached command string, if present
pub fn get_cached_response(input: &str) -> Option<String> {
    if let Ok(mut cache) = RESPONSE_CACHE.lock() { cache.get(input).cloned() } else { None }
}

/// Store a response in the cache
///
/// # Arguments
///
/// * `input` - The sanitized input string to use as the cache key
/// * `response` - The command string to cache
pub fn store_response(input: &str, response: &str) {
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.put(input.to_string(), response.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let input = "test input";
        let response = "ducktape calendar create \"Test\" 2025-04-23 14:00 15:00 \"Work\"";

        // Store in cache
        store_response(input, response);

        // Retrieve from cache
        let cached = get_cached_response(input);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), response);

        // Check missing entry
        let missing = get_cached_response("does not exist");
        assert!(missing.is_none());
    }
}
