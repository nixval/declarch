//! Cache clean command
//!
//! Cleans package manager caches for configured backends.

use crate::backends::load_all_backends_unified;
use crate::packages::traits::PackageManager;
use crate::error::Result;
use crate::commands::runtime_overrides::{
    apply_runtime_backend_overrides, load_runtime_config_for_command,
};
use crate::ui as output;
use std::collections::HashSet;

pub struct CacheOptions {
    /// Target specific backends (None = all backends)
    pub backends: Option<Vec<String>>,
    /// Verbose output
    pub verbose: bool,
}

/// Run cache clean for configured backends
pub fn run(options: CacheOptions) -> Result<()> {
    // Load all configured backends (import-based or legacy)
    let all_backends = load_all_backends_unified()?;

    if all_backends.is_empty() {
        output::warning("No backends configured");
        output::info("Run 'declarch init --backend <name>' to add a backend");
        return Ok(());
    }

    // Filter backends if specific ones requested
    let mut backends_to_clean: Vec<_> = match &options.backends {
        Some(target_backends) => {
            let target_set: HashSet<_> = target_backends.iter().cloned().collect();
            let selected: Vec<_> = all_backends
                .into_iter()
                .filter(|(name, _)| target_set.contains(name))
                .collect();

            let selected_names: HashSet<_> =
                selected.iter().map(|(name, _)| name.clone()).collect();
            let mut unknown: Vec<_> = target_set
                .into_iter()
                .filter(|name| !selected_names.contains(name))
                .collect();
            unknown.sort();
            if !unknown.is_empty() {
                output::warning(&format!("Unknown backend(s): {}", unknown.join(", ")));
            }

            selected
        }
        None => all_backends.into_iter().collect(),
    };
    backends_to_clean.sort_by(|a, b| a.0.cmp(&b.0));

    if backends_to_clean.is_empty() {
        output::warning("No matching backends found");
        return Ok(());
    }

    let runtime_config = load_runtime_config_for_command("cache command");

    // First pass: check which backends can clean cache
    let mut cleanable_backends = Vec::new();
    let mut skipped_no_cmd = Vec::new();
    let mut skipped_not_available = Vec::new();

    for (name, mut config) in backends_to_clean {
        apply_runtime_backend_overrides(&mut config, &name, &runtime_config);

        if config.cache_clean_cmd.is_none() {
            skipped_no_cmd.push(name);
            continue;
        }
        
        let manager: Box<dyn PackageManager> = Box::new(crate::backends::GenericManager::from_config(
            config,
            crate::core::types::Backend::from(name.as_str()),
            false,
        ));
        if manager.is_available() && manager.supports_cache_clean() {
            cleanable_backends.push((name, manager));
        } else if !manager.is_available() {
            skipped_not_available.push(name);
        } else if options.verbose {
            output::warning(&format!("Skipped '{}': cache clean not supported", name));
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
