//! Regex compilation cache for performance optimization
//!
//! This module provides a thread-safe cache for compiled regex patterns,
//! avoiding expensive recompilation when the same pattern is used multiple times.

use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Global cache for compiled regex patterns
static REGEX_CACHE: LazyLock<Mutex<HashMap<String, Regex>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get or compile a regex pattern from the cache
///
/// This function checks the global cache first and returns a cloned regex
/// if it exists. Otherwise, it compiles the pattern, stores it in the cache,
/// and returns the compiled regex.
///
/// # Arguments
/// * `pattern` - The regex pattern string
///
/// # Returns
/// * `Ok(Regex)` - The compiled regex (either cached or newly compiled)
/// * `Err(regex::Error)` - If the pattern is invalid
///
/// # Examples
/// ```
/// use declarch::utils::regex_cache::get_cached_regex;
///
/// let regex = get_cached_regex(r"\d+").unwrap();
/// assert!(regex.is_match("123"));
/// ```
pub fn get_cached_regex(pattern: &str) -> Result<Regex, regex::Error> {
    // Try to get from cache first
    if let Ok(cache) = REGEX_CACHE.lock()
        && let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }

    // Compile new regex
    let regex = Regex::new(pattern)?;

    // Store in cache (ignore lock poisoning)
    if let Ok(mut cache) = REGEX_CACHE.lock() {
        cache.insert(pattern.to_string(), regex.clone());
    }

    Ok(regex)
}

/// Check if a pattern is already cached
///
/// # Arguments
/// * `pattern` - The regex pattern string
///
/// # Returns
/// `true` if the pattern is in the cache, `false` otherwise
pub fn is_cached(pattern: &str) -> bool {
    REGEX_CACHE
        .lock()
        .map(|cache| cache.contains_key(pattern))
        .unwrap_or(false)
}

/// Clear the regex cache
///
/// This removes all cached patterns from memory. It's primarily useful
/// for testing to ensure a clean state between tests.
pub fn clear_cache() {
    if let Ok(mut cache) = REGEX_CACHE.lock() {
        cache.clear();
    }
}

/// Get the number of cached patterns
///
/// # Returns
/// The number of patterns currently in the cache
pub fn cache_size() -> usize {
    REGEX_CACHE
        .lock()
        .map(|cache| cache.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that regex compilation works
    #[test]
    fn test_regex_compilation() {
        let pattern = r"\d+";
        let regex = get_cached_regex(pattern).unwrap();
        assert!(regex.is_match("123"));
        assert!(!regex.is_match("abc"));
    }

    /// Test that patterns are cached after first use
    #[test]
    fn test_caching() {
        // Use a pattern unlikely to be used elsewhere
        let pattern = r"test_caching_abc123_\w+";
        
        // First call
        let _ = get_cached_regex(pattern).unwrap();
        
        // Should now be cached
        assert!(is_cached(pattern));
    }

    /// Test that invalid patterns return an error
    #[test]
    fn test_invalid_regex() {
        let pattern = r"[invalid(";
        let result = get_cached_regex(pattern);
        assert!(result.is_err());
        assert!(!is_cached(pattern));
    }

    /// Test cache clear functionality
    #[test]
    fn test_clear_clears_all() {
        // Add a pattern
        let pattern = r"test_clear_\d+";
        get_cached_regex(pattern).unwrap();
        
        // Verify it's cached
        assert!(is_cached(pattern));
        
        // Clear cache
        clear_cache();
        
        // Verify it's gone
        assert!(!is_cached(pattern));
    }
}
