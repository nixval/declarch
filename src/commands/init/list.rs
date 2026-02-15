//! List available backends and modules from the registry
//!
//! Provides static lists of available backends and modules.
//! In the future, these could be fetched dynamically from the registry API.

use crate::error::Result;
use crate::ui as output;
use colored::Colorize;

/// Backend info tuple: (name, description, tags)
type BackendInfo<'a> = (&'a str, &'a str, Vec<&'a str>);

/// Module info tuple: (path, description, tags)
type ModuleInfo<'a> = (&'a str, &'a str, Vec<&'a str>);

/// List available backends from the registry
pub fn list_available_backends() -> Result<()> {
    output::header("Available Backends");

    // Static list of known backends in the registry
    // This could be fetched dynamically from the registry API in the future
    let backends: Vec<BackendInfo> = vec![
        (
            "apt",
            "Debian/Ubuntu package manager",
            vec!["debian", "ubuntu", "linux"],
        ),
        (
            "aur",
            "Meta backend for AUR (paru→yay→pacman fallback)",
            vec!["arch", "linux", "meta"],
        ),
        (
            "brew",
            "Homebrew for macOS/Linux",
            vec!["macos", "linux", "homebrew"],
        ),
        (
            "bun",
            "Fast JavaScript toolkit",
            vec!["nodejs", "javascript", "fast"],
        ),
        (
            "cargo",
            "Rust package manager",
            vec!["rust", "build-system"],
        ),
        (
            "dnf",
            "Fedora/RHEL package manager",
            vec!["fedora", "rhel", "linux"],
        ),
        (
            "flatpak",
            "Universal Linux apps",
            vec!["linux", "universal", "desktop"],
        ),
        ("gem", "Ruby package manager", vec!["ruby", "gems"]),
        ("go", "Go package installer", vec!["go", "golang"]),
        (
            "nix",
            "Nix functional package manager",
            vec!["nix", "nixos", "functional"],
        ),
        (
            "npm",
            "Node.js package manager",
            vec!["nodejs", "javascript"],
        ),
        (
            "pacman",
            "Arch Linux native package manager",
            vec!["arch", "linux", "native"],
        ),
        (
            "paru",
            "Feature-packed AUR helper",
            vec!["arch", "linux", "aur", "rust"],
        ),
        ("pip", "Python package installer", vec!["python", "pypi"]),
        (
            "pnpm",
            "Fast, disk space efficient package manager",
            vec!["nodejs", "javascript", "fast"],
        ),
        (
            "snap",
            "Universal packages by Canonical",
            vec!["linux", "universal", "canonical"],
        ),
        (
            "soar",
            "Static binary package manager",
            vec!["linux", "static", "fast"],
        ),
        (
            "yarn",
            "Fast, reliable dependency management",
            vec!["nodejs", "javascript"],
        ),
        (
            "yay",
            "Yet Another Yogurt AUR helper",
            vec!["arch", "linux", "aur", "go"],
        ),
    ];

    println!();
    const SYSTEM_PMS: &[&str] = &[
        "apt",
        "aur",
        "dnf",
        "pacman",
        "paru",
        "yay",
        "flatpak",
        "snap",
        "nix",
        "soar",
        "brew",
    ];
    const LANG_PMS: &[&str] = &["npm", "yarn", "pnpm", "bun", "pip", "cargo", "gem", "go"];

    println!("{}", "System Package Managers:".bold().cyan());
    for (name, desc, _) in &backends {
        if SYSTEM_PMS.contains(name) {
            println!("  {} - {}", name.bold(), desc.dimmed());
        }
    }

    println!();
    println!("{}", "Language-Specific Package Managers:".bold().cyan());
    for (name, desc, _) in &backends {
        if LANG_PMS.contains(name) {
            println!("  {} - {}", name.bold(), desc.dimmed());
        }
    }

    println!();
    output::info("To initialize a backend:");
    println!(
        "  {}        Initialize single backend",
        "declarch init --backend <name>".green()
    );
    println!(
        "  {}  Initialize multiple backends",
        "declarch init --backend a,b,c".green()
    );
    println!(
        "  {}        Initialize with auto-yes",
        "declarch init --backend <name> -y".green()
    );

    Ok(())
}

/// Check if a module exists in the registry
pub fn is_module_available(name: &str) -> bool {
    let modules: Vec<ModuleInfo> = vec![
        (
            "system/base",
            "Essential packages for any Linux system",
            vec!["base", "essential", "core"],
        ),
        (
            "desktop/hyprland",
            "Hyprland Wayland compositor setup",
            vec!["desktop", "wayland", "hyprland"],
        ),
        (
            "apps/flatpak-common",
            "Common GUI applications via Flatpak",
            vec!["apps", "gui", "flatpak"],
        ),
        (
            "development/nodejs",
            "Node.js development environment",
            vec!["dev", "nodejs", "javascript"],
        ),
        (
            "dev/rust",
            "Rust development tools",
            vec!["dev", "rust", "cargo"],
        ),
        (
            "apps/multimedia",
            "Multimedia applications",
            vec!["apps", "media", "audio", "video"],
        ),
    ];

    modules.iter().any(|(path, _, _)| *path == name)
}

/// List available modules from the registry
pub fn list_available_modules() -> Result<()> {
    output::header("Available Modules");

    // Static list of known modules in the registry
    // This could be fetched dynamically from the registry API in the future
    let modules: Vec<ModuleInfo> = vec![
        (
            "system/base",
            "Essential packages for any Linux system",
            vec!["base", "essential", "core"],
        ),
        (
            "desktop/hyprland",
            "Hyprland Wayland compositor setup",
            vec!["desktop", "wayland", "hyprland"],
        ),
        (
            "apps/flatpak-common",
            "Common GUI applications via Flatpak",
            vec!["apps", "gui", "flatpak"],
        ),
        (
            "development/nodejs",
            "Node.js development environment",
            vec!["dev", "nodejs", "javascript"],
        ),
        (
            "dev/rust",
            "Rust development tools",
            vec!["dev", "rust", "cargo"],
        ),
        (
            "apps/multimedia",
            "Multimedia applications",
            vec!["apps", "media", "audio", "video"],
        ),
    ];

    // Group by category
    let mut by_category: std::collections::HashMap<&str, Vec<&ModuleInfo>> =
        std::collections::HashMap::new();
    for m in &modules {
        let category = m.0.split('/').next().unwrap_or("other");
        by_category.entry(category).or_default().push(m);
    }

    println!();
    for (category, cat_modules) in by_category {
        println!("{}", format!("{}/", category).bold().cyan());
        for (name, desc, tags) in cat_modules {
            let module_name = name.split('/').nth(1).unwrap_or(name);
            println!(
                "  {} - {} {}",
                module_name.bold(),
                desc.dimmed(),
                format!("[{}]", tags.join(", ")).purple().dimmed()
            );
        }
        println!();
    }

    output::info("To initialize a module:");
    println!(
        "  {}     Initialize from registry",
        "declarch init <category>/<name>".green()
    );
    println!(
        "  {}          Create local module",
        "declarch init --local <name>".green()
    );
    println!(
        "  {}             Auto-confirm import",
        "declarch init <module> -y".green()
    );

    Ok(())
}
