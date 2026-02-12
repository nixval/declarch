//! Cache clean command
//!
//! Cleans package manager caches for configured backends.

use crate::backends::load_all_backends;
use crate::config::types::GlobalConfig;
use crate::error::Result;
use crate::packages::create_manager;
use crate::ui as output;

pub struct CacheOptions {
    /// Target specific backends (None = all backends)
    pub backends: Option<Vec<String>>,
    /// Verbose output
    pub verbose: bool,
}

/// Run cache clean for configured backends
pub fn run(options: CacheOptions) -> Result<()> {
    // Load all configured backends
    let all_backends = load_all_backends()?;

    if all_backends.is_empty() {
        output::warning("No backends configured");
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Filter backends if specific ones requested
    let backends_to_clean: Vec<_> = match &options.backends {
        Some(target_backends) => all_backends
            .into_iter()
            .filter(|(name, _)| target_backends.contains(name))
            .collect(),
        None => all_backends.into_iter().collect(),
    };

    if backends_to_clean.is_empty() {
        output::warning("No matching backends found");
        return Ok(());
    }

    let global_config = GlobalConfig::default();
    
    // First pass: check which backends can clean cache
    let mut cleanable_backends = Vec::new();
    let mut skipped_no_cmd = Vec::new();
    let mut skipped_not_available = Vec::new();

    for (name, config) in backends_to_clean {
        if config.cache_clean_cmd.is_none() {
            skipped_no_cmd.push(name);
            continue;
        }
        
        match create_manager(&crate::core::types::Backend::from(name.as_str()), &global_config, false) {
            Ok(manager) => {
                if manager.is_available() && manager.supports_cache_clean() {
                    cleanable_backends.push((name, manager));
                } else if !manager.is_available() {
                    skipped_not_available.push(name);
                }
            }
            Err(e) => {
                if options.verbose {
                    output::warning(&format!("Failed to initialize '{}': {}", name, e));
                }
                skipped_not_available.push(name);
            }
        }
    }

    // Show compact summary of skipped backends
    if !skipped_no_cmd.is_empty() {
        output::warning(&format!(
            "Skipped (no cache_clean_cmd): {}",
            skipped_no_cmd.join(", ")
        ));
    }
    if !skipped_not_available.is_empty() {
        output::warning(&format!(
            "Skipped (not available): {}",
            skipped_not_available.join(", ")
        ));
    }

    if cleanable_backends.is_empty() {
        output::info("No backends to clean");
        return Ok(());
    }

    // Clean backends
    output::info("Cleaning caches...");
    output::separator();

    let mut cleaned_count = 0;
    let mut failed_count = 0;

    for (name, manager) in cleanable_backends {
        match manager.clean_cache() {
            Ok(()) => {
                cleaned_count += 1;
            }
            Err(e) => {
                if options.verbose {
                    output::warning(&format!("Failed to clean '{}': {}", name, e));
                    output::info(&format!("  Recommendation: Check backend configuration or run with --force to ignore errors"));
                } else {
                    output::warning(&format!("Failed to clean '{}' (use --verbose for details)", name));
                }
                failed_count += 1;
            }
        }
    }

    output::separator();
    if cleaned_count > 0 {
        output::success(&format!("Cleaned {} backend cache(s)", cleaned_count));
    }
    if failed_count > 0 {
        output::warning(&format!("Failed {} backend(s)", failed_count));
    }

    Ok(())
}
