use crate::utils::{output, errors::Result};
use crate::state;
use crate::state::types::Backend;
use colored::Colorize;

pub fn run() -> Result<()> {
    // Load State
    let state = state::io::load_state()?;

    output::header("System Status");
    output::keyval("Hostname", &state.meta.hostname.cyan().bold().to_string());
    output::keyval("Last Sync", &state.meta.last_sync.format("%Y-%m-%d %H:%M:%S").to_string());

    let pkg_count = state.packages.len();
    let aur_count = state.packages.values().filter(|p| p.backend == Backend::Aur).count();
    let flatpak_count = state.packages.values().filter(|p| p.backend == Backend::Flatpak).count();

    println!();
    output::tag("Total Managed", &pkg_count.to_string());
    output::indent(&format!("• AUR/Repo: {}", aur_count), 2);
    output::indent(&format!("• Flatpak:  {}", flatpak_count), 2);

    if pkg_count > 0 {
        output::separator();
        println!("{}", "Managed Packages:".bold());
        
        let mut sorted_packages: Vec<_> = state.packages.iter().collect();
        sorted_packages.sort_by_key(|(name, _)| *name);

        for (name, pkg_state) in sorted_packages {
            let backend_tag = match pkg_state.backend {
                Backend::Aur => "aur".blue(),
                Backend::Flatpak => "flt".green(),
            };
            
            println!("  {} {} {}", 
                backend_tag,
                "→".dimmed(),
                name
            );
        }
    }

    Ok(())
}
