use crate::ui as output;
use crate::error::Result;
use crate::state;
use colored::Colorize;

pub fn run() -> Result<()> {
    let state = state::io::load_state()?;

    output::header("System Status");
    output::keyval("Hostname", &state.meta.hostname.cyan().bold().to_string());
    output::keyval("Last Sync", &state.meta.last_sync.format("%Y-%m-%d %H:%M:%S").to_string());

    let pkg_count = state.packages.len();
    
    // Count logic needs to parse the new Keys or iterate values
    let aur_count = state.packages.values().filter(|p| matches!(p.backend, crate::state::types::Backend::Aur)).count();
    let flatpak_count = state.packages.values().filter(|p| matches!(p.backend, crate::state::types::Backend::Flatpak)).count();

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
             let n1 = k1.split_once(':').map(|(_,n)| n).unwrap_or(k1);
             let n2 = k2.split_once(':').map(|(_,n)| n).unwrap_or(k2);
             n1.cmp(n2)
        });

        for (key, pkg_state) in sorted_packages {
            // Extract pure name for display
            let name = key.split_once(':').map(|(_,n)| n).unwrap_or(key);

            match &pkg_state.backend {
                crate::state::types::Backend::Aur => {
                    // Requested: Remove 'aur' prefix/tag for native packages
                    println!("  {} {}",
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Flatpak => {
                    println!("  {} {} {}",
                        "flt".green(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Soar => {
                    println!("  {} {} {}",
                        "soar".blue(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Npm => {
                    println!("  {} {} {}",
                        "npm".cyan(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Yarn => {
                    println!("  {} {} {}",
                        "yarn".cyan(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Pnpm => {
                    println!("  {} {} {}",
                        "pnpm".cyan(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Bun => {
                    println!("  {} {} {}",
                        "bun".cyan(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Pip => {
                    println!("  {} {} {}",
                        "pip".blue(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Cargo => {
                    println!("  {} {} {}",
                        "cargo".red(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Brew => {
                    println!("  {} {} {}",
                        "brew".purple(),
                        "→".dimmed(),
                        name
                    );
                },
                crate::state::types::Backend::Custom(backend_name) => {
                    println!("  {} {} {}",
                        backend_name.white().dimmed(),
                        "→".dimmed(),
                        name
                    );
                },
            };
        }
    }

    Ok(())
}
