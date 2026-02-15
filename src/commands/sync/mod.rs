//! Sync command orchestration
//!
//! This module orchestrates the sync workflow by coordinating:
//! - Transaction planning (planner.rs)
//! - Package installation/removal (executor.rs)
//! - State updates (state_sync.rs)
//! - Hook execution (hooks.rs)
//! - Variant matching (variants.rs)

mod executor;
mod hooks;
mod planner;
mod state_sync;
mod variants;

// Re-export public API
pub use executor::execute_transaction;
pub use hooks::{
    execute_on_failure, execute_on_success, execute_on_update, execute_post_sync, execute_pre_sync,
};
pub use planner::{
    check_variant_transitions, create_transaction, display_transaction_plan, warn_partial_upgrade,
};
pub use state_sync::{update_state, update_state_with_success};
pub use variants::{find_variant, resolve_installed_package_name};

use crate::config::loader;
use crate::core::types::SyncTarget;
use crate::error::Result;
use crate::ui as output;
use crate::utils::paths;
use std::path::Path;

use crate::core::types::{PackageId, PackageMetadata};
use crate::packages::PackageManager;
use crate::state;
use crate::state::types::Backend;
use std::collections::HashMap;

// Re-export dry-run display function
pub use planner::display_dry_run_details;

// Type aliases to reduce complexity
pub type InstalledSnapshot = HashMap<PackageId, PackageMetadata>;
pub type ManagerMap = HashMap<Backend, Box<dyn PackageManager>>;

#[derive(Debug)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub prune: bool,
    pub gc: bool,
    pub update: bool,
    pub yes: bool,
    pub force: bool,
    pub target: Option<String>,
    pub noconfirm: bool,
    pub hooks: bool,
    pub profile: Option<String>,
    pub host: Option<String>,
    pub modules: Vec<String>,
    pub diff: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
}

pub fn run(options: SyncOptions) -> Result<()> {
    // Acquire exclusive lock at the very beginning to prevent concurrent sync
    // Lock is held until this function returns (RAII pattern)
    let lock = if options.dry_run {
        // Dry-run doesn't need exclusive lock, but we check if another process is running
        match state::io::acquire_lock() {
            Ok(lock) => Some(lock),
            Err(_) => {
                output::warning(
                    "Another declarch process is running. Dry-run may show stale state.",
                );
                None
            }
        }
    } else {
        // Real sync requires exclusive lock
        Some(state::io::acquire_lock().map_err(|e| {
            crate::error::DeclarchError::Other(format!(
                "Cannot start sync: {}\n\
                 If no other declarch process is running, delete the lock file manually.",
                e
            ))
        })?)
    };

    // 1. Load Config
    let config_path = paths::config_file()?;
    let selectors = loader::LoadSelectors {
        profile: options.profile.clone(),
        host: options.host.clone(),
    };

    let mut config = if !options.modules.is_empty() {
        if options.modules.len() == 1 && options.target.is_none() {
            load_single_module(&config_path, &options.modules[0], &selectors)?
        } else {
            load_config_with_modules(&config_path, &options.modules, &selectors)?
        }
    } else {
        loader::load_root_config_with_selectors(&config_path, &selectors)?
    };
    enforce_sync_policy(&config)?;
    let hooks_enabled = resolve_hooks_enabled(&config, &options);

    // 2. Target Resolution
    let sync_target = resolve_target(&options.target, &config);
    if let SyncTarget::Named(query) = &sync_target
        && !named_target_exists(&config, query)
    {
        return Err(crate::error::DeclarchError::Other(format!(
            "No package or module matched target '{}'",
            query
        )));
    }

    // Execute pre-sync hooks
    execute_pre_sync(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;

    // 3. Initialize Managers & Snapshot
    let (installed_snapshot, managers) =
        initialize_managers_and_snapshot(&config, &options, &sync_target)?;

    // 3.5. Run backend updates if --update flag is set
    if options.update && !options.dry_run {
        execute_backend_updates(&managers)?;
        execute_on_update(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;
    }

    // 4. Load State & Resolve
    let state = state::io::load_state()?;

    // 5. Create Transaction
    let transaction = create_transaction(
        &mut config,
        &state,
        &installed_snapshot,
        &managers,
        &sync_target,
    )?;

    // 5.5 Check for dangerous variant transitions and warn about stale updates
    check_variant_transitions(
        &config,
        &installed_snapshot,
        &state,
        &transaction,
        &sync_target,
        &options,
    )?;
    warn_partial_upgrade(&state, &transaction, &options);

    // 6. Display Plan
    if transaction.to_install.is_empty()
        && transaction.to_prune.is_empty()
        && transaction.to_adopt.is_empty()
    {
        output::success("Everything is up to date!");
        execute_post_sync(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;
        execute_on_success(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;
        return Ok(());
    }

    // Show detailed dry-run info or regular plan
    if options.dry_run {
        display_dry_run_details(&transaction, options.prune, &installed_snapshot);
    } else {
        display_transaction_plan(&transaction, options.prune);
    }

    // Handle --diff flag: Show diff and exit (like git diff)
    if options.diff {
        show_sync_diff(&transaction, &installed_snapshot);
        return Ok(());
    }

    // 7. Execute
    if !options.dry_run {
        if !options.yes && !output::prompt_yes_no("Proceed with sync?") {
            output::info("Sync cancelled");
            return Err(crate::error::DeclarchError::Interrupted);
        }

        let successfully_installed =
            match execute_transaction(&transaction, &managers, &config, &options, hooks_enabled) {
                Ok(installed) => installed,
                Err(e) => {
                    let _ = execute_on_failure(
                        &config.lifecycle_actions,
                        hooks_enabled,
                        options.dry_run,
                    );
                    return Err(e);
                }
            };

        // 8. Refresh installed snapshot and update state with successful packages
        let post_execution_snapshot = refresh_installed_snapshot(&managers);

        let new_state = update_state_with_success(
            &state,
            &transaction,
            &post_execution_snapshot,
            &options,
            &successfully_installed,
        )?;

        // Save state with lock held (ensures no concurrent modifications)
        if let Some(ref lock) = lock {
            if let Err(e) = state::io::save_state_locked(&new_state, lock) {
                let _ =
                    execute_on_failure(&config.lifecycle_actions, hooks_enabled, options.dry_run);
                return Err(e);
            }
        } else {
            // This shouldn't happen for non-dry-run, but handle gracefully
            if let Err(e) = state::io::save_state(&new_state) {
                let _ =
                    execute_on_failure(&config.lifecycle_actions, hooks_enabled, options.dry_run);
                return Err(e);
            }
        }
    } else {
        // Dry-run complete
        output::success("Dry-run completed - no changes were made");
    }

    // Execute post-sync hooks
    execute_post_sync(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;
    execute_on_success(&config.lifecycle_actions, hooks_enabled, options.dry_run)?;

    Ok(())
}

fn resolve_hooks_enabled(config: &loader::MergedConfig, options: &SyncOptions) -> bool {
    if !options.hooks {
        return false;
    }

    if config
        .policy
        .as_ref()
        .and_then(|p| p.forbid_hooks)
        .unwrap_or(false)
    {
        output::warning("Hooks are blocked by policy { forbid_hooks true }.");
        return false;
    }

    if config.is_experimental_enabled("enable-hooks") {
        return true;
    }

    output::warning(
        "Hooks were requested but blocked by policy. Add experimental { \"enable-hooks\" } to declarch.kdl to allow hook execution.",
    );
    false
}

fn enforce_sync_policy(config: &loader::MergedConfig) -> Result<()> {
    let Some(policy) = config.policy.as_ref() else {
        return Ok(());
    };

    if policy.require_backend.unwrap_or(false) {
        let legacy_default: Vec<_> = config
            .packages
            .keys()
            .filter(|pkg| pkg.backend.to_string() == "default")
            .map(|pkg| pkg.name.clone())
            .collect();

        if !legacy_default.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: require-backend=true but {} package(s) still use legacy default backend: {}",
                legacy_default.len(),
                legacy_default.join(", ")
            )));
        }
    }

    if policy.duplicate_is_error() {
        let duplicates = config.get_duplicates();
        if !duplicates.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: on-duplicate=error and {} duplicate declaration(s) were found",
                duplicates.len()
            )));
        }
    }

    if policy.conflict_is_error() {
        let conflicts = config.get_cross_backend_conflicts();
        if !conflicts.is_empty() {
            return Err(crate::error::DeclarchError::ConfigError(format!(
                "Policy violation: on-conflict=error and {} cross-backend conflict(s) were found",
                conflicts.len()
            )));
        }
    }

    Ok(())
}

/// Show diff view of sync changes
fn show_sync_diff(
    transaction: &crate::core::resolver::Transaction,
    installed_snapshot: &InstalledSnapshot,
) {
    use colored::Colorize;

    output::header("Sync Diff");

    // Show packages to install
    if !transaction.to_install.is_empty() {
        println!("\n{}:", "Packages to install".green().bold());
        for pkg_id in &transaction.to_install {
            println!("  {} {} {}", "+".green(), pkg_id.backend, pkg_id.name);
        }
    }

    // Show packages to remove
    if !transaction.to_prune.is_empty() {
        println!("\n{}:", "Packages to remove".red().bold());
        for pkg_id in &transaction.to_prune {
            let version = installed_snapshot
                .get(pkg_id)
                .and_then(|m| m.version.as_ref())
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            println!(
                "  {} {} {}{}",
                "-".red(),
                pkg_id.backend,
                pkg_id.name,
                version.dimmed()
            );
        }
    }

    // Show packages to adopt
    if !transaction.to_adopt.is_empty() {
        println!("\n{}:", "Packages to adopt".yellow().bold());
        for pkg_id in &transaction.to_adopt {
            println!("  {} {} {}", "~".yellow(), pkg_id.backend, pkg_id.name);
        }
    }

    // Summary
    println!();
    let total_changes =
        transaction.to_install.len() + transaction.to_prune.len() + transaction.to_adopt.len();
    output::info(&format!("Total changes: {}", total_changes));
    output::info("Run 'declarch sync' to apply these changes");
}

fn resolve_target(target: &Option<String>, config: &loader::MergedConfig) -> SyncTarget {
    if let Some(t) = target {
        let normalized_backend = Backend::from(t.as_str());
        let matches_backend_in_packages = config
            .packages
            .keys()
            .any(|pkg_id| pkg_id.backend == normalized_backend);
        let matches_backend_in_imports = config
            .backends
            .iter()
            .any(|backend| backend.name.eq_ignore_ascii_case(t));

        if matches_backend_in_packages || matches_backend_in_imports {
            SyncTarget::Backend(normalized_backend)
        } else {
            SyncTarget::Named(t.clone())
        }
    } else {
        SyncTarget::All
    }
}

fn named_target_exists(config: &loader::MergedConfig, query: &str) -> bool {
    let query_lower = query.to_lowercase();

    for (pkg_id, sources) in &config.packages {
        if pkg_id.name == query {
            return true;
        }

        for source in sources {
            if let Some(stem) = source.file_stem()
                && stem.to_string_lossy().to_lowercase() == query_lower
            {
                return true;
            }
        }
    }

    false
}

fn initialize_managers_and_snapshot(
    config: &loader::MergedConfig,
    options: &SyncOptions,
    sync_target: &SyncTarget,
) -> Result<(InstalledSnapshot, ManagerMap)> {
    let mut installed_snapshot: InstalledSnapshot = HashMap::new();
    let mut managers: ManagerMap = HashMap::new();

    let mut known_backends = crate::backends::load_all_backends_unified()?;
    for backend in &config.backends {
        known_backends.insert(backend.name.clone(), backend.clone());
    }

    // Get backends from config (unique set)
    let configured_backends: std::collections::HashSet<Backend> = config
        .packages
        .keys()
        .map(|pkg_id| pkg_id.backend.clone())
        .collect();

    // Initialize managers for configured backends
    for backend in configured_backends {
        let backend_name = backend.name().to_string();
        let Some(mut backend_config) = known_backends.get(&backend_name).cloned() else {
            output::warning(&format!(
                "Backend '{}' is referenced by packages but has no config. Run 'declarch init --backend {}'",
                backend_name, backend_name
            ));
            continue;
        };

        apply_backend_option_overrides(&mut backend_config, &backend_name, config);
        apply_backend_env_overrides(&mut backend_config, &backend_name, config);
        apply_backend_package_sources(&mut backend_config, &backend_name, config);

        if !crate::utils::platform::backend_supports_current_os(&backend_config) {
            let current_os = crate::utils::platform::current_os_tag();
            let supported = crate::utils::platform::supported_os_summary(&backend_config);
            output::warning(&format!(
                "Skipping backend '{}' on this device (current OS: {}, supported: {}).",
                backend_name, current_os, supported
            ));
            output::info("This is okay. Keep it in your config for other machines.");
            continue;
        }

        let manager: Box<dyn PackageManager> =
            Box::new(crate::backends::GenericManager::from_config(
                backend_config,
                backend.clone(),
                options.noconfirm,
            ));

        let available = manager.is_available();

        if !available && matches!(sync_target, SyncTarget::Backend(b) if b == &backend) {
            output::warning(&format!(
                "Backend '{}' is not available on this system.",
                backend
            ));
        }

        if available {
            match manager.list_installed() {
                Ok(packages) => {
                    for (name, meta) in packages {
                        let pkg_id = PackageId {
                            name: name.clone(),
                            backend: backend.clone(),
                        };
                        installed_snapshot.insert(pkg_id, meta);
                    }
                }
                Err(e) => {
                    output::warning(&format!("Failed to list packages for {}: {}", backend, e));
                }
            }
            managers.insert(backend.clone(), manager);
        }
    }

    Ok((installed_snapshot, managers))
}

fn parse_bool_option(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub(crate) fn apply_backend_option_overrides(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let Some(options) = config.backend_options.get(backend_name) else {
        return;
    };

    for (key, value) in options {
        let normalized = value.trim();
        let disable = normalized == "-";

        match key.as_str() {
            "noconfirm_flag" => {
                backend_config.noconfirm_flag = if disable { None } else { Some(value.clone()) }
            }
            "fallback" => {
                backend_config.fallback = if disable { None } else { Some(value.clone()) }
            }
            "install_cmd" => {
                if disable {
                    output::warning(&format!(
                        "Ignoring invalid disable sentinel for required option options:{} -> install_cmd=-",
                        backend_name
                    ));
                } else if !normalized.contains("{packages}") {
                    output::warning(&format!(
                        "Ignoring invalid install_cmd override for options:{}: missing '{{packages}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.install_cmd = value.clone();
                }
            }
            "remove_cmd" => {
                if disable {
                    backend_config.remove_cmd = None;
                } else if !normalized.contains("{packages}") {
                    output::warning(&format!(
                        "Ignoring invalid remove_cmd override for options:{}: missing '{{packages}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.remove_cmd = Some(value.clone());
                }
            }
            "list_cmd" => {
                backend_config.list_cmd = if disable { None } else { Some(value.clone()) }
            }
            "search_cmd" => {
                if disable {
                    backend_config.search_cmd = None;
                } else if !normalized.contains("{query}") {
                    output::warning(&format!(
                        "Ignoring invalid search_cmd override for options:{}: missing '{{query}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.search_cmd = Some(value.clone());
                }
            }
            "search_local_cmd" => {
                if disable {
                    backend_config.search_local_cmd = None;
                } else if !normalized.contains("{query}") {
                    output::warning(&format!(
                        "Ignoring invalid search_local_cmd override for options:{}: missing '{{query}}' placeholder",
                        backend_name
                    ));
                } else {
                    backend_config.search_local_cmd = Some(value.clone());
                }
            }
            "update_cmd" => {
                backend_config.update_cmd = if disable { None } else { Some(value.clone()) }
            }
            "cache_clean_cmd" => {
                backend_config.cache_clean_cmd = if disable { None } else { Some(value.clone()) }
            }
            "upgrade_cmd" => {
                backend_config.upgrade_cmd = if disable { None } else { Some(value.clone()) }
            }
            "needs_sudo" | "sudo" => {
                if let Some(parsed) = parse_bool_option(value) {
                    backend_config.needs_sudo = parsed;
                } else {
                    output::warning(&format!(
                        "Invalid boolean for options:{} -> {}={}",
                        backend_name, key, value
                    ));
                }
            }
            _ => {
                output::warning(&format!(
                    "Unknown backend option ignored: options:{} -> {}",
                    backend_name, key
                ));
            }
        }
    }
}

pub(crate) fn apply_backend_env_overrides(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let mut merged_env: HashMap<String, String> =
        backend_config.preinstall_env.clone().unwrap_or_default();

    for scope in ["global", backend_name] {
        if let Some(vars) = config.env.get(scope) {
            for var in vars {
                if let Some((k, v)) = var.split_once('=') {
                    merged_env.insert(k.trim().to_string(), v.trim().to_string());
                } else {
                    output::warning(&format!(
                        "Ignoring invalid env entry in env:{} -> {}",
                        scope, var
                    ));
                }
            }
        }
    }

    if merged_env.is_empty() {
        backend_config.preinstall_env = None;
    } else {
        backend_config.preinstall_env = Some(merged_env);
    }
}

pub(crate) fn apply_backend_package_sources(
    backend_config: &mut crate::backends::config::BackendConfig,
    backend_name: &str,
    config: &loader::MergedConfig,
) {
    let mut sources = backend_config
        .package_sources
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if let Some(repo_sources) = config.package_sources.get(backend_name) {
        for src in repo_sources {
            let trimmed = src.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !sources.iter().any(|s| s == trimmed) {
                sources.push(trimmed.to_string());
            }
        }
    }

    if sources.is_empty() {
        backend_config.package_sources = None;
    } else {
        backend_config.package_sources = Some(sources);
    }
}

fn refresh_installed_snapshot(managers: &ManagerMap) -> InstalledSnapshot {
    let mut snapshot = InstalledSnapshot::new();
    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }
        match manager.list_installed() {
            Ok(packages) => {
                for (name, meta) in packages {
                    snapshot.insert(
                        PackageId {
                            name,
                            backend: backend.clone(),
                        },
                        meta,
                    );
                }
            }
            Err(e) => {
                output::warning(&format!(
                    "Failed to refresh package snapshot for {}: {}",
                    backend, e
                ));
            }
        }
    }
    snapshot
}

/// Execute update for all backends that support it
fn execute_backend_updates(managers: &ManagerMap) -> Result<()> {
    output::separator();
    output::info("Updating package indices...");

    let mut updated_count = 0;
    let mut skipped_count = 0;

    for (backend, manager) in managers {
        if !manager.is_available() {
            continue;
        }

        if !manager.supports_update() {
            output::info(&format!("Skipping '{}': no update_cmd configured", backend));
            skipped_count += 1;
            continue;
        }

        match manager.update() {
            Ok(()) => {
                updated_count += 1;
            }
            Err(e) => {
                output::warning(&format!("Failed to update '{}': {}", backend, e));
                skipped_count += 1;
            }
        }
    }

    if updated_count > 0 {
        output::info(&format!("Updated {} backend(s)", updated_count));
    }
    if skipped_count > 0 {
        output::info(&format!("Skipped {} backend(s)", skipped_count));
    }

    Ok(())
}

fn load_single_module(
    _config_path: &Path,
    module_name: &str,
    selectors: &loader::LoadSelectors,
) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    let module_path = paths::module_file(module_name);

    let final_path = if let Ok(path) = module_path {
        if path.exists() {
            path
        } else {
            let direct_path = PathBuf::from(module_name);
            if direct_path.exists() {
                direct_path
            } else {
                return Err(crate::error::DeclarchError::Other(format!(
                    "Module not found: {}",
                    module_name
                )));
            }
        }
    } else {
        let direct_path = PathBuf::from(module_name);
        if direct_path.exists() {
            direct_path
        } else {
            return Err(crate::error::DeclarchError::Other(format!(
                "Module not found: {}",
                module_name
            )));
        }
    };

    let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
    Ok(module_config)
}

fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
    selectors: &loader::LoadSelectors,
) -> Result<loader::MergedConfig> {
    use std::path::PathBuf;

    let mut merged = loader::load_root_config_with_selectors(config_path, selectors)?;

    for module_name in extra_modules {
        let module_path = paths::module_file(module_name);

        let final_path = if let Ok(path) = module_path {
            if path.exists() {
                path
            } else {
                let direct_path = PathBuf::from(module_name);
                if direct_path.exists() {
                    direct_path
                } else {
                    return Err(crate::error::DeclarchError::Other(format!(
                        "Module not found: {}",
                        module_name
                    )));
                }
            }
        } else {
            let direct_path = PathBuf::from(module_name);
            if direct_path.exists() {
                direct_path
            } else {
                return Err(crate::error::DeclarchError::Other(format!(
                    "Module not found: {}",
                    module_name
                )));
            }
        };

        output::info(&format!("  Loading module: {}", final_path.display()));
        let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::config::BackendConfig;
    use crate::core::types::PackageId;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn merged_config_with_options(
        backend_name: &str,
        options: &[(&str, &str)],
    ) -> loader::MergedConfig {
        let mut merged = loader::MergedConfig::default();
        let mut backend_opts = HashMap::new();
        for (k, v) in options {
            backend_opts.insert((*k).to_string(), (*v).to_string());
        }
        merged
            .backend_options
            .insert(backend_name.to_string(), backend_opts);
        merged
    }

    #[test]
    fn test_parse_bool_option_variants() {
        assert_eq!(parse_bool_option("true"), Some(true));
        assert_eq!(parse_bool_option("yes"), Some(true));
        assert_eq!(parse_bool_option("on"), Some(true));
        assert_eq!(parse_bool_option("1"), Some(true));
        assert_eq!(parse_bool_option("false"), Some(false));
        assert_eq!(parse_bool_option("no"), Some(false));
        assert_eq!(parse_bool_option("off"), Some(false));
        assert_eq!(parse_bool_option("0"), Some(false));
        assert_eq!(parse_bool_option("maybe"), None);
    }

    #[test]
    fn test_backend_option_overrides_disable_sentinel() {
        let mut backend = BackendConfig {
            name: "paru".to_string(),
            install_cmd: "paru -S {packages}".to_string(),
            remove_cmd: Some("paru -R {packages}".to_string()),
            list_cmd: Some("paru -Q".to_string()),
            search_cmd: Some("paru -Ss {query}".to_string()),
            search_local_cmd: Some("paru -Q {query}".to_string()),
            update_cmd: Some("paru -Sy".to_string()),
            cache_clean_cmd: Some("paru -Sc".to_string()),
            upgrade_cmd: Some("paru -Syu".to_string()),
            fallback: Some("pacman".to_string()),
            noconfirm_flag: Some("--noconfirm".to_string()),
            needs_sudo: true,
            ..Default::default()
        };

        let merged = merged_config_with_options(
            "paru",
            &[
                ("install_cmd", "-"),
                ("remove_cmd", "-"),
                ("list_cmd", "-"),
                ("search_cmd", "-"),
                ("search_local_cmd", "-"),
                ("update_cmd", "-"),
                ("cache_clean_cmd", "-"),
                ("upgrade_cmd", "-"),
                ("fallback", "-"),
                ("noconfirm_flag", "-"),
                ("needs_sudo", "invalid"),
                ("unknown_key", "value"),
            ],
        );

        apply_backend_option_overrides(&mut backend, "paru", &merged);

        assert_eq!(backend.install_cmd, "paru -S {packages}");
        assert!(backend.remove_cmd.is_none());
        assert!(backend.list_cmd.is_none());
        assert!(backend.search_cmd.is_none());
        assert!(backend.search_local_cmd.is_none());
        assert!(backend.update_cmd.is_none());
        assert!(backend.cache_clean_cmd.is_none());
        assert!(backend.upgrade_cmd.is_none());
        assert!(backend.fallback.is_none());
        assert!(backend.noconfirm_flag.is_none());
        assert!(backend.needs_sudo);
    }

    #[test]
    fn test_backend_option_overrides_apply_valid_values() {
        let mut backend = BackendConfig {
            name: "pacman".to_string(),
            install_cmd: "pacman -S {packages}".to_string(),
            remove_cmd: Some("pacman -R {packages}".to_string()),
            needs_sudo: false,
            ..Default::default()
        };

        let merged = merged_config_with_options(
            "pacman",
            &[
                ("remove_cmd", "pacman -Rns {packages}"),
                ("update_cmd", "pacman -Sy"),
                ("needs_sudo", "on"),
                ("noconfirm_flag", "--noconfirm"),
            ],
        );

        apply_backend_option_overrides(&mut backend, "pacman", &merged);

        assert_eq!(
            backend.remove_cmd.as_deref(),
            Some("pacman -Rns {packages}")
        );
        assert_eq!(backend.update_cmd.as_deref(), Some("pacman -Sy"));
        assert_eq!(backend.noconfirm_flag.as_deref(), Some("--noconfirm"));
        assert!(backend.needs_sudo);
    }

    #[test]
    fn test_backend_option_overrides_reject_invalid_templates() {
        let mut backend = BackendConfig {
            name: "paru".to_string(),
            install_cmd: "paru -S {packages}".to_string(),
            remove_cmd: Some("paru -R {packages}".to_string()),
            search_cmd: Some("paru -Ss {query}".to_string()),
            search_local_cmd: Some("paru -Q {query}".to_string()),
            ..Default::default()
        };

        let merged = merged_config_with_options(
            "paru",
            &[
                ("install_cmd", "paru -S"),
                ("remove_cmd", "paru -R"),
                ("search_cmd", "paru -Ss"),
                ("search_local_cmd", "paru -Q"),
            ],
        );

        apply_backend_option_overrides(&mut backend, "paru", &merged);

        assert_eq!(backend.install_cmd, "paru -S {packages}");
        assert_eq!(backend.remove_cmd.as_deref(), Some("paru -R {packages}"));
        assert_eq!(backend.search_cmd.as_deref(), Some("paru -Ss {query}"));
        assert_eq!(backend.search_local_cmd.as_deref(), Some("paru -Q {query}"));
    }

    #[test]
    fn test_resolve_target_detects_backend_from_imports() {
        let mut merged = loader::MergedConfig::default();
        merged.backends.push(BackendConfig {
            name: "paru".to_string(),
            ..Default::default()
        });

        let target = resolve_target(&Some("paru".to_string()), &merged);
        match target {
            SyncTarget::Backend(b) => assert_eq!(b.name(), "paru"),
            _ => panic!("expected backend target"),
        }
    }

    #[test]
    fn test_named_target_exists_by_package_or_module_stem() {
        let mut merged = loader::MergedConfig::default();
        let pkg = PackageId {
            name: "bat".to_string(),
            backend: Backend::from("paru"),
        };
        merged
            .packages
            .insert(pkg, vec![PathBuf::from("/tmp/devtools.kdl")]);

        assert!(named_target_exists(&merged, "bat"));
        assert!(named_target_exists(&merged, "devtools"));
        assert!(!named_target_exists(&merged, "unknown"));
    }

    #[test]
    fn test_apply_backend_package_sources_normalizes_and_dedupes() {
        let mut backend = BackendConfig {
            name: "paru".to_string(),
            package_sources: Some(vec!["core".to_string(), " extra ".to_string()]),
            ..Default::default()
        };
        let mut merged = loader::MergedConfig::default();
        merged.package_sources.insert(
            "paru".to_string(),
            vec![
                "extra".to_string(),
                "multilib".to_string(),
                "   ".to_string(),
                "core".to_string(),
            ],
        );

        apply_backend_package_sources(&mut backend, "paru", &merged);

        assert_eq!(
            backend.package_sources,
            Some(vec![
                "core".to_string(),
                "extra".to_string(),
                "multilib".to_string()
            ])
        );
    }
}
