//! Diff display for sync transactions
//!
//! Provides git-diff-like output for previewing package changes

use crate::core::resolver::Transaction;
use crate::state::types::Backend;
use colored::Colorize;

/// Display transaction as a git-style diff
pub fn display_transaction_diff(tx: &Transaction, prune: bool) {
    // Group packages by backend for better readability
    let mut install_by_backend: std::collections::HashMap<Backend, Vec<&str>> =
        std::collections::HashMap::new();
    let mut adopt_by_backend: std::collections::HashMap<Backend, Vec<&str>> =
        std::collections::HashMap::new();
    let mut prune_by_backend: std::collections::HashMap<Backend, Vec<&str>> =
        std::collections::HashMap::new();

    // Group to_install by backend
    for pkg in &tx.to_install {
        install_by_backend
            .entry(pkg.backend.clone())
            .or_default()
            .push(&pkg.name);
    }

    // Group to_adopt by backend
    for pkg in &tx.to_adopt {
        adopt_by_backend
            .entry(pkg.backend.clone())
            .or_default()
            .push(&pkg.name);
    }

    // Group to_prune by backend
    for pkg in &tx.to_prune {
        prune_by_backend
            .entry(pkg.backend.clone())
            .or_default()
            .push(&pkg.name);
    }

    let has_changes = !tx.to_install.is_empty()
        || !tx.to_adopt.is_empty()
        || (prune && !tx.to_prune.is_empty())
        || !tx.to_update_project_metadata.is_empty();

    if !has_changes {
        println!("{}", "No changes".bright_black());
        return;
    }

    println!("{}", "diff --declarch".bright_black());
    println!("{}", "--- Current State".red());
    println!("{}", "+++ Desired State".green());
    println!();

    // Show packages to be installed (+)
    if !tx.to_install.is_empty() {
        println!(
            "{}",
            format!("@@ Install {} packages @@", tx.to_install.len()).cyan()
        );
        for (backend, packages) in &install_by_backend {
            println!(
                "{}  [{}]",
                "+".green().bold(),
                format!("{:?}", backend).yellow()
            );
            for pkg in packages {
                println!("{}    {}", "+".green(), pkg.green());
            }
        }
        println!();
    }

    // Show packages to be adopted (~)
    if !tx.to_adopt.is_empty() {
        println!(
            "{}",
            format!("@@ Adopt {} packages @@", tx.to_adopt.len()).cyan()
        );
        for (backend, packages) in &adopt_by_backend {
            println!(
                "{}  [{}]",
                "~".yellow().bold(),
                format!("{:?}", backend).yellow()
            );
            for pkg in packages {
                println!("{}    {}", "~".yellow(), pkg.yellow());
            }
        }
        println!();
    }

    // Show packages to be removed (-) if prune is enabled
    if prune && !tx.to_prune.is_empty() {
        println!(
            "{}",
            format!("@@ Remove {} packages @@", tx.to_prune.len()).cyan()
        );
        for (backend, packages) in &prune_by_backend {
            println!(
                "{}  [{}]",
                "-".red().bold(),
                format!("{:?}", backend).yellow()
            );
            for pkg in packages {
                println!("{}    {}", "-".red(), pkg.red());
            }
        }
        println!();
    }

    // Show packages to update metadata (M)
    if !tx.to_update_project_metadata.is_empty() {
        println!(
            "{}",
            format!(
                "@@ Update {} package metadata @@",
                tx.to_update_project_metadata.len()
            )
            .cyan()
        );
        println!("{}  Project metadata updates", "M".blue().bold());
        for pkg in &tx.to_update_project_metadata {
            println!(
                "{}    {} [{}]",
                "M".blue(),
                pkg.name,
                format!("{:?}", pkg.backend).yellow()
            );
        }
        println!();
    }

    // Summary
    let total_add = tx.to_install.len() + tx.to_adopt.len();
    let total_remove = if prune { tx.to_prune.len() } else { 0 };
    let total_update = tx.to_update_project_metadata.len();

    println!(
        "{}",
        format!(
            "Summary: {} added, {} removed, {} modified",
            total_add, total_remove, total_update
        )
        .bright_black()
    );
}
