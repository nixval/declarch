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
}

/// Run cache clean for configured backends
pub fn run(options: CacheOptions) -> Result<()> {
    output::info("Loading backend configurations...");

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
    let mut cleaned_count = 0;
    let mut skipped_count = 0;

    for (name, config) in backends_to_clean {
        // Check if backend has cache_clean_cmd configured
        if config.cache_clean_cmd.is_none() {
            output::info(&format!(
                "Skipping '{}': no cache_clean_cmd configured",
                name
            ));
            skipped_count += 1;
            continue;
        }

        // Create manager for this backend
        match create_manager(
            &crate::core::types::Backend::from(name.as_str()),
            &global_config,
            false,
        ) {
            Ok(manager) => {
                if !manager.is_available() {
                    output::warning(&format!("Backend '{}' is not available", name));
                    skipped_count += 1;
                    continue;
                }

                if !manager.supports_cache_clean() {
                    output::warning(&format!(
                        "Backend '{}' does not support cache cleaning",
                        name
                    ));
                    skipped_count += 1;
                    continue;
                }

                match manager.clean_cache() {
                    Ok(()) => {
                        cleaned_count += 1;
                    }
                    Err(e) => {
                        output::warning(&format!(
                            "Failed to clean cache for '{}': {}",
                            name, e
                        ));
                        skipped_count += 1;
                    }
                }
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to create manager for '{}': {}",
                    name, e
                ));
                skipped_count += 1;
            }
        }
    }

    output::separator();
    if cleaned_count > 0 {
        output::success(&format!("Cleaned {} backend cache(s)", cleaned_count));
    }
    if skipped_count > 0 {
        output::info(&format!("Skipped {} backend(s)", skipped_count));
    }

    Ok(())
}
