//! Summary display for declarch info
//!
//! Provides git-status-like quick overview of system state

use crate::config::loader;
use crate::error::Result;
use crate::state;
use colored::Colorize;
use std::collections::HashMap;

/// Display a git-status-like summary of package state
pub fn display_summary() -> Result<()> {
    let state = state::io::load_state()?;
    let config_path = crate::utils::paths::config_file()?;

    // Load config to compare
    let config = if config_path.exists() {
        loader::load_root_config(&config_path).ok()
    } else {
        None
    };

    // Calculate statistics
    let total_managed = state.packages.len();
    let last_sync = state.meta.last_sync;
    let days_since_sync = (chrono::Utc::now() - last_sync).num_days();

    // Count packages by backend
    let mut backend_counts: HashMap<String, usize> = HashMap::new();
    for pkg in state.packages.values() {
        let backend_str = format!("{:?}", pkg.backend).to_lowercase();
        *backend_counts.entry(backend_str).or_insert(0) += 1;
    }

    // Calculate drift (packages in config but not in state)
    let mut drift_install = 0;
    let mut drift_remove = 0;

    if let Some(ref cfg) = config {
        // Packages in config but not in state = need install
        drift_install = cfg
            .packages
            .keys()
            .filter(|pkg_id| {
                let key = format!("{:?}:{}", pkg_id.backend, pkg_id.name);
                !state.packages.contains_key(&key)
            })
            .count();

        // Packages in state but not in config = orphans
        let config_keys: std::collections::HashSet<String> = cfg
            .packages
            .keys()
            .map(|id| format!("{:?}:{}", id.backend, id.name))
            .collect();
        drift_remove = state
            .packages
            .keys()
            .filter(|key| !config_keys.contains(*key))
            .count();
    }

    // Header
    println!("{}", "On branch".bright_black());
    println!(
        "{} {}",
        "Host:".bright_black(),
        state.meta.hostname.cyan().bold()
    );
    println!();

    // Sync status
    if days_since_sync > 7 {
        println!(
            "{} Last sync was {} days ago",
            "⚠".yellow(),
            days_since_sync
        );
    } else {
        println!("{} Last sync: {} day(s) ago", "✓".green(), days_since_sync);
    }
    println!();

    // Changes to be committed (drift)
    if drift_install > 0 || drift_remove > 0 {
        println!("{}", "Changes not staged for sync:".yellow().bold());
        if drift_install > 0 {
            println!("  {} {} packages to install", "+".green(), drift_install);
        }
        if drift_remove > 0 {
            println!("  {} {} packages to remove", "-".red(), drift_remove);
        }
        println!();
        println!("{} {}", "Run:".bright_black(), "declarch sync".cyan());
        println!();
    }

    // Summary statistics
    println!("{}", "Managed packages:".bold());
    println!("  Total: {}", total_managed.to_string().cyan().bold());

    // Show backend breakdown
    let ordered_backends = vec!["aur", "flatpak", "soar", "npm", "cargo", "pip"];
    for backend in &ordered_backends {
        if let Some(&count) = backend_counts.get(*backend)
            && count > 0
        {
            println!("  {}: {}", backend.cyan(), count);
        }
    }

    // Untracked files (orphans)
    if drift_remove > 0 {
        println!();
        println!(
            "{}",
            format!("{} orphan packages not tracked", drift_remove).red()
        );
        println!("  (use \"declarch list orphans\" to see them)");
        println!("  (use \"declarch sync prune\" to remove them)");
    }

    Ok(())
}
