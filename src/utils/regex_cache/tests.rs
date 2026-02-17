
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
