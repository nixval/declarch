use crate::config::loader;
use crate::error::Result;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use colored::Colorize;
use std::process::Command;

pub fn run(doctor: bool, debug: bool) -> Result<()> {
    // Handle --debug flag (must be set early)
    if debug {
        unsafe { std::env::set_var("RUST_LOG", "debug") };
        output::info("Debug logging enabled");
    }

    // Handle --doctor flag
    if doctor {
        return run_doctor();
    }

    // Default info behavior
    run_info()
}

fn run_info() -> Result<()> {
    let state = state::io::load_state()?;

    output::header("System Status");
    output::keyval("Hostname", &state.meta.hostname.cyan().bold().to_string());
    output::keyval(
        "Last Sync",
        &state.meta.last_sync.format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    let pkg_count = state.packages.len();

    // Count logic needs to parse the new Keys or iterate values
    let aur_count = state
        .packages
        .values()
        .filter(|p| matches!(p.backend, crate::state::types::Backend::Aur))
        .count();
    let flatpak_count = state
        .packages
        .values()
        .filter(|p| matches!(p.backend, crate::state::types::Backend::Flatpak))
        .count();

    println!();
    output::tag("Total Managed", &pkg_count.to_string());
    output::indent(&format!("• AUR/Repo: {}", aur_count), 2);
    output::indent(&format!("• Flatpak:  {}", flatpak_count), 2);

    if pkg_count > 0 {
        output::separator();
        println!("{}", "Managed Packages:".bold());

        // Sort by name (need to extract name from key "backend:name")
        let mut sorted_packages: Vec<_> = state.packages.iter().collect();
        sorted_packages.sort_by(|(k1, _), (k2, _)| {
            let n1 = k1.split_once(':').map(|(_, n)| n).unwrap_or(k1);
            let n2 = k2.split_once(':').map(|(_, n)| n).unwrap_or(k2);
            n1.cmp(n2)
        });

        for (key, pkg_state) in sorted_packages {
            // Extract pure name for display
            let name = key.split_once(':').map(|(_, n)| n).unwrap_or(key);

            match &pkg_state.backend {
                crate::state::types::Backend::Aur => {
                    println!("  {} {}", "→".dimmed(), name);
                }
                crate::state::types::Backend::Flatpak => {
                    println!("  {} {} {}", "flt".green(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Soar => {
                    println!("  {} {} {}", "soar".blue(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Npm => {
                    println!("  {} {} {}", "npm".cyan(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Yarn => {
                    println!("  {} {} {}", "yarn".cyan(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Pnpm => {
                    println!("  {} {} {}", "pnpm".cyan(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Bun => {
                    println!("  {} {} {}", "bun".cyan(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Pip => {
                    println!("  {} {} {}", "pip".blue(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Cargo => {
                    println!("  {} {} {}", "cargo".red(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Brew => {
                    println!("  {} {} {}", "brew".purple(), "→".dimmed(), name);
                }
                crate::state::types::Backend::Custom(backend_name) => {
                    println!(
                        "  {} {} {}",
                        backend_name.white().dimmed(),
                        "→".dimmed(),
                        name
                    );
                }
            };
        }
    }

    Ok(())
}

fn run_doctor() -> Result<()> {
    output::header("System Diagnosis");

    let mut all_ok = true;

    // Check 1: Config file
    output::info("Checking configuration file...");
    let config_path = paths::config_file()?;
    if config_path.exists() {
        output::success(&format!("Config found: {}", config_path.display()));

        // Try to parse it
        match loader::load_root_config(&config_path) {
            Ok(config) => {
                output::success(&format!("Config valid: {} packages defined", config.packages.len()));
            }
            Err(e) => {
                output::error(&format!("Config parse error: {}", e));
                all_ok = false;
            }
        }
    } else {
        output::warning(&format!("Config not found: {}", config_path.display()));
        output::info("Run 'dcl init' to create a configuration");
        all_ok = false;
    }

    // Check 2: State file
    output::info("Checking state file...");
    let state_path = state::io::get_state_path()?;
    if state_path.exists() {
        output::success(&format!("State found: {}", state_path.display()));

        // Try to load it
        match state::io::load_state() {
            Ok(state) => {
                output::success(&format!("State valid: {} packages tracked", state.packages.len()));

                // Check for orphans
                if config_path.exists() {
                    if let Ok(config) = loader::load_root_config(&config_path) {
                        use crate::core::types::PackageId;
                        use std::collections::HashSet;

                        let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
                        let mut orphan_count = 0;

                        for (_key, pkg_state) in &state.packages {
                            let pkg_id = PackageId {
                                backend: pkg_state.backend.clone(),
                                name: pkg_state.config_name.clone(),
                            };
                            if !config_set.contains(&pkg_id) {
                                orphan_count += 1;
                            }
                        }

                        if orphan_count > 0 {
                            output::warning(&format!("Found {} orphan packages (not in config)", orphan_count));
                            output::info("Run 'dcl list --orphans' to see them");
                            output::info("Run 'dcl sync --prune' to remove orphans");
                        } else {
                            output::success("No orphan packages found");
                        }
                    }
                }
            }
            Err(e) => {
                output::error(&format!("State load error: {}", e));
                all_ok = false;
            }
        }
    } else {
        output::warning("State not found");
        output::info("Run 'dcl sync' to create initial state");
    }

    // Check 3: Backend availability
    output::info("Checking backends...");
    let backends_to_check = vec![
        ("paru", "AUR (paru)"),
        ("yay", "AUR (yay)"),
        ("flatpak", "Flatpak"),
        ("cargo", "Cargo"),
        ("npm", "npm"),
        ("bun", "Bun"),
        ("pip", "pip"),
    ];

    let mut available_backends = 0;
    for (cmd, name) in &backends_to_check {
        if is_command_available(cmd) {
            output::success(&format!("{}: Available", name));
            available_backends += 1;
        } else {
            output::warning(&format!("{}: Not found", name));
        }
    }

    // Check 4: State consistency
    output::info("Checking state consistency...");
    if state_path.exists() {
        if let Ok(state) = state::io::load_state() {
            // Check for duplicate keys
            let keys: Vec<_> = state.packages.keys().collect();
            let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
            if keys.len() != unique_keys.len() {
                output::warning("State has duplicate keys - consider running sync to fix");
            } else {
                output::success("State consistency: OK");
            }

            // Check last sync time
            let now = chrono::Utc::now();
            let days_since_sync = (now - state.meta.last_sync).num_days();
            if days_since_sync > 7 {
                output::warning(&format!("Last sync was {} days ago", days_since_sync));
                output::info("Consider running 'dcl sync' to update");
            } else {
                output::success(&format!("Last sync: {} day(s) ago", days_since_sync));
            }
        }
    }

    // Summary
    output::separator();
    if all_ok {
        output::success("All checks passed!");
        output::info(&format!("Available backends: {}/{}", available_backends, backends_to_check.len()));
    } else {
        output::warning("Some issues found - see details above");
    }

    Ok(())
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
