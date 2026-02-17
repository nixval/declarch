//! Backend template generation
//!
//! This module contains templates for generating default backend configurations.
//! Templates are used when creating new backends via `declarch init --backend <name>`.
use crate::project_identity;

/// Generate a backend template based on name
///
/// Returns pre-defined templates for known backends (aur, pacman, flatpak)
/// or generates a generic template for custom backends.
pub fn generate_backend_template(name: &str) -> String {
    match name {
        "aur" => generate_aur_backend_template(),
        "pacman" => generate_pacman_backend_template(),
        "flatpak" => generate_flatpak_backend_template(),
        _ => generate_generic_backend_template(name),
    }
}

/// AUR backend template with fallback chain
#[allow(dead_code)]
pub fn generate_aur_backend_template() -> String {
    r#"// aur - AUR Helper with Fallback Chain
// 
// This backend provides a complete fallback chain:
//   aur (alias) → paru → yay → pacman
//
// Behavior:
//   1. If 'paru' is installed, use it (best AUR helper)
//   2. Else if 'yay' is installed, use it (alternative AUR helper)
//   3. Else if 'pacman' backend exists, fallback to it (official repos only)
//
// Installation:
//   paru:  pacman -S paru      (or from AUR: paru-bin)
//   yay:   pacman -S yay       (or from AUR: yay-bin)

backend "aur" {
    meta {
        title "AUR Helper"
        description "Arch User Repository with automatic fallback chain"
        kdl-maintainer "declarch"
        tags "package-manager" "aur" "arch" "fallback"
        platforms "arch"
        requires "paru" "yay" "pacman"
    }
    
    // Binary fallback chain: try paru first, then yay
    binary "paru" "yay"
    
    // List all installed packages (official repo + AUR)
    // Output format: "package-name version"
    list "{binary} -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    // Install packages (supports both AUR and official repos)
    install "{binary} -S --needed {packages}"
    
    // Remove packages
    remove "{binary} -R {packages}"
    
    // Search packages in AUR and repos
    search "{binary} -Ss {query}" {
        format "whitespace"
        name_col 0
        desc_col 1
    }
    
    // If no AUR helper available, fallback to pacman
    fallback "pacman"
}
"#
    .replace(
        r#"kdl-maintainer "declarch""#,
        &format!(
            r#"kdl-maintainer "{}""#,
            project_identity::STABLE_PROJECT_ID
        ),
    )
}

/// Pacman backend template
#[allow(dead_code)]
pub fn generate_pacman_backend_template() -> String {
    r#"// pacman - Arch Linux Native Package Manager
//
// This is the ultimate fallback when no AUR helper is available.
// It only supports official repositories, not AUR.

backend "pacman" {
    meta {
        title "Pacman"
        description "Arch Linux native package manager (official repos only)"
        kdl-maintainer "declarch"
        tags "package-manager" "arch" "native" "official"
        platforms "arch"
        requires "pacman"
    }
    
    binary "pacman"
    
    list "pacman -Q" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "pacman -S --needed {packages}"
    remove "pacman -R {packages}"
    
    search "pacman -Ss {query}" {
        format "whitespace"
        name_col 0
        desc_col 1
    }
    
    needs_sudo "true"
}
"#
    .replace(
        r#"kdl-maintainer "declarch""#,
        &format!(
            r#"kdl-maintainer "{}""#,
            project_identity::STABLE_PROJECT_ID
        ),
    )
}

/// Flatpak backend template
#[allow(dead_code)]
pub fn generate_flatpak_backend_template() -> String {
    r#"// flatpak - Universal Linux Application Sandboxing
//
// Flatpak runs applications in isolated sandboxes with their own dependencies.
// Applications are identified by reverse DNS: com.vendor.AppName

backend "flatpak" {
    meta {
        title "Flatpak"
        description "Universal Linux application sandboxing"
        kdl-maintainer "declarch"
        tags "package-manager" "flatpak" "sandbox" "universal" "gui"
        platforms "linux"
        requires "flatpak"
    }
    
    binary "flatpak"
    
    // List installed applications
    // Output: Application ID    Version    Branch    Installation
    list "flatpak list --app --columns=application,version" {
        format "tsv"
        name_col 0
        version_col 1
    }
    
    // Install from flathub (default remote)
    install "flatpak install flathub {packages}"
    
    // Remove applications
    remove "flatpak uninstall {packages}"
    
    // Search flathub
    search "flatpak search {query}" {
        format "tsv"
        name_col 2
        desc_col 1
    }
    
    // Auto-confirm operations
    noconfirm "-y"
    
    // Flatpak doesn't need sudo for user installations
    needs_sudo "false"
}
"#
    .replace(
        r#"kdl-maintainer "declarch""#,
        &format!(
            r#"kdl-maintainer "{}""#,
            project_identity::STABLE_PROJECT_ID
        ),
    )
}

/// Generic backend template for custom backends
#[allow(dead_code)]
pub fn generate_generic_backend_template(name: &str) -> String {
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    format!(
        r#"// {name} - Custom Backend Configuration
// 
// This is a template for the '{name}' package manager.
// Customize the commands below to match your package manager's syntax.
//
// Common placeholders:
//   {{packages}}  - Space-separated package names
//   {{query}}     - Search query (for search command)
//
// Output formats:
//   - whitespace: Space-separated columns (default)
//   - tsv:        Tab-separated values
//   - json:       JSON output with configurable path
//   - regex:      Regular expression pattern matching

backend "{name}" {{
    meta {{
        title "{name}"
        description "Custom {name} backend configuration"
        kdl-maintainer "user"
        tags "package-manager" "{name}"
        created "{date}"
        platforms "linux"
        requires "{name}"
    }}
    
    // Binary to use (can specify multiple for fallback)
    binary "{name}"
    
    // Command to list installed packages
    //
    // Example outputs and formats:
    // - "package version"        → format "whitespace", name_col 0, version_col 1
    // - "package\tversion"       → format "tsv", name_col 0, version_col 1
    // - '{{"packages":[{{"name":"x"}}]}}' → format "json", json {{ path "packages" name_key "name" }}
    list "{name} list" {{
        format "whitespace"
        name_col 0
        // version_col 1  // Uncomment if output includes version
    }}
    
    // Install command
    // {{packages}} will be replaced with space-separated package names
    install "{name} install {{packages}}"
    
    // Remove/uninstall command
    remove "{name} remove {{packages}}"
    
    // Search command (optional but recommended)
    // Uncomment and customize if your package manager supports search
    // search "{name} search {{query}}" {{
    //     format "whitespace"
    //     name_col 0
    //     desc_col 1
    // }}
    
    // Auto-confirmation flag (optional)
    // Uncomment if your package manager has a "yes to all" flag
    // noconfirm "-y"
    
    // Whether this backend requires sudo (default: false)
    // Uncomment if install/remove need root privileges
    // needs_sudo "true"
    
    // Fallback backend (optional)
    // If this backend's binary is not found, use another backend instead
    // fallback "alternative-backend"
}}
"#,
        name = name,
        date = current_date
    )
}
