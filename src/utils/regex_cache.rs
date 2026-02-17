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
        && let Some(regex) = cache.get(pattern)
    {
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
    REGEX_CACHE.lock().map(|cache| cache.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests;
