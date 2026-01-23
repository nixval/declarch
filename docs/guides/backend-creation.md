# Adding New Package Manager Backends

This guide explains how to add new package manager backends to declarch, making it easy to extend support for additional package managers like nala, nix, snap, cargo, etc.

## Architecture Overview

Declarch uses a **modular backend architecture** that makes adding new package managers straightforward:

- **Backend Registry**: Central registry for managing available backends
- **PackageManager Trait**: Common interface all backends must implement
- **Distro Detection**: Automatic backend availability based on the system
- **Config Parser**: Flexible KDL-based configuration for each backend

## Step-by-Step Guide

### 1. Add Backend Variant

Add the new backend to `src/core/types.rs`:

```rust
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Aur,
    Flatpak,
    Soar,
    Nala,     // NEW: Debian/Ubuntu backend
    // Future: Nix, Snap, Cargo, etc.
}
```

Update the `Display` implementation:

```rust
impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aur => write!(f, "aur"),
            Self::Flatpak => write!(f, "flatpak"),
            Self::Soar => write!(f, "soar"),
            Self::Nala => write!(f, "nala"),  // NEW
        }
    }
}
```

Update `PackageId` display (choose an appropriate prefix or no prefix):

```rust
impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.backend {
            Backend::Aur => write!(f, "{}", self.name),
            Backend::Flatpak => write!(f, "flatpak:{}", self.name),
            Backend::Soar => write!(f, "{}", self.name),
            Backend::Nala => write!(f, "{}", self.name),  // NEW: no prefix
        }
    }
}
```

Update `FromStr` implementation (add optional prefix support):

```rust
impl FromStr for PackageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(name) = s.strip_prefix("flatpak:") {
            Ok(PackageId { name: name.to_string(), backend: Backend::Flatpak })
        } else if let Some(name) = s.strip_prefix("soar:") {
            Ok(PackageId { name: name.to_string(), backend: Backend::Soar })
        } else if let Some(name) = s.strip_prefix("nala:") {  // NEW
            Ok(PackageId { name: name.to_string(), backend: Backend::Nala })
        } else if let Some(name) = s.strip_prefix("aur:") {
            Ok(PackageId { name: name.to_string(), backend: Backend::Aur })
        } else {
            // Default to AUR/Native if no prefix provided
            Ok(PackageId { name: s.to_string(), backend: Backend::Aur })
        }
    }
}
```

### 2. Implement PackageManager Trait

Create `src/packages/nala.rs`:

```rust
use crate::packages::traits::PackageManager;
use crate::core::types::{Backend, PackageMetadata};
use crate::error::{DeclarchError, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use chrono::Utc;

pub struct NalaManager {
    pub noconfirm: bool,
}

impl NalaManager {
    pub fn new(noconfirm: bool) -> Self {
        Self { noconfirm }
    }

    /// Check if nala command is available
    fn is_nala_installed() -> bool {
        std::path::Path::new("/usr/bin/nala").exists()
            || Command::new("which")
                .arg("nala")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }
}

impl PackageManager for NalaManager {
    fn backend_type(&self) -> Backend {
        Backend::Nala
    }

    fn is_available(&self) -> bool {
        Self::is_nala_installed()
    }

    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // Use nala list or dpkg-query to get installed packages
        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Package}\t${Version}\n"])
            .output()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "dpkg-query".into(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(DeclarchError::PackageManagerError(
                "Failed to query nala database".into()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut installed = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if let Some(name) = parts.get(0) {
                let version = parts.get(1).map(|&v| v.to_string());

                installed.insert(name.to_string(), PackageMetadata {
                    version,
                    installed_at: Utc::now(),
                    source_file: None,
                });
            }
        }

        Ok(installed)
    }

    fn install(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new("nala");
        cmd.arg("install");

        if self.noconfirm {
            cmd.arg("-y");
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "nala install".into(),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut cmd = Command::new("nala");
        cmd.arg("remove");

        if self.noconfirm {
            cmd.arg("-y");
        }

        cmd.args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| DeclarchError::SystemCommandFailed {
                command: "nala remove".into(),
                reason: e.to_string(),
            })?;

        Ok(())
    }
}
```

### 3. Register in Backend Registry

Update `src/packages/mod.rs`:

```rust
pub mod traits;
pub mod aur;
pub mod flatpak;
pub mod soar;
pub mod nala;     // NEW
pub mod registry;

pub use traits::PackageManager;
pub use registry::{BackendRegistry, get_registry, create_manager};
```

Update `src/packages/registry.rs` in the `register_defaults` method:

```rust
pub fn register_defaults(&mut self) {
    // AUR Backend (Arch Linux)
    self.register(Backend::Aur, |config, noconfirm| {
        Ok(Box::new(crate::packages::aur::AurManager::new(
            config.aur_helper.to_string(),
            noconfirm,
        )))
    });

    // Flatpak Backend (Cross-distro)
    self.register(Backend::Flatpak, |_config, noconfirm| {
        Ok(Box::new(crate::packages::flatpak::FlatpakManager::new(noconfirm)))
    });

    // Soar Backend (Cross-distro)
    self.register(Backend::Soar, |_config, noconfirm| {
        Ok(Box::new(crate::packages::soar::SoarManager::new(noconfirm)))
    });

    // Nala Backend (Debian-based)  // NEW
    self.register(Backend::Nala, |_config, noconfirm| {
        Ok(Box::new(crate::packages::nala::NalaManager::new(noconfirm)))
    });
}
```

Update `available_backends` method to handle distro-specific availability:

```rust
pub fn available_backends(&self, distro: &DistroType) -> Vec<Backend> {
    let mut backends = Vec::new();

    for backend in self.registered_backends() {
        match backend {
            Backend::Aur => {
                // Only AUR on Arch-based systems
                if distro.supports_aur() {
                    backends.push(backend);
                }
            }
            Backend::Nala => {  // NEW
                // Only Nala on Debian-based systems
                if matches!(distro, DistroType::Debian) {
                    backends.push(backend);
                }
            }
            Backend::Soar | Backend::Flatpak => {
                // These work on all distros
                backends.push(backend);
            }
        }
    }

    backends
}
```

### 4. Update Config Parser

Add support for your backend in `src/config/kdl.rs`:

```rust
#[derive(Debug, Clone)]
pub struct RawConfig {
    pub imports: Vec<String>,
    pub packages: Vec<String>,           // Soar
    pub aur_packages: Vec<String>,        // AUR
    pub flatpak_packages: Vec<String>,    // Flatpak
    pub nala_packages: Vec<String>,       // NEW: Nala
    pub excludes: Vec<String>,
    pub aliases: HashMap<String, String>,
}
```

Update the parser:

```rust
pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse()?;

    let mut config = RawConfig {
        imports: vec![],
        packages: vec![],
        aur_packages: vec![],
        flatpak_packages: vec![],
        nala_packages: vec![],  // NEW
        excludes: vec![],
        aliases: HashMap::new(),
    };

    for node in doc.nodes() {
        match node.name().value() {
            "import" | "imports" => {
                extract_strings(node, &mut config.imports);
            },
            "packages" | "package" => {
                extract_mixed_values(node, &mut config.packages);
            },
            "aur-packages" | "aur-package" => {
                extract_mixed_values(node, &mut config.aur_packages);
            },
            "nala-packages" | "nala-package" => {  // NEW
                extract_mixed_values(node, &mut config.nala_packages);
            },
            "flatpak-packages" | "flatpak-package" => {
                extract_mixed_values(node, &mut config.flatpak_packages);
            },
            "exclude" | "excludes" => {
                extract_mixed_values(node, &mut config.excludes);
            },
            "aliases-pkg" | "alias-pkg" => {
                extract_aliases(node, &mut config.aliases);
            },
            _ => {}
        }
    }

    Ok(config)
}
```

Update `src/config/loader.rs` to handle the new packages:

```rust
// Process Nala packages (Debian-only)
if matches!(distro, DistroType::Debian) {
    for pkg_str in raw.nala_packages {
        let pkg_id = PackageId {
            name: pkg_str,
            backend: Backend::Nala,
        };

        merged.packages.entry(pkg_id)
            .or_default()
            .push(canonical_path.clone());
    }
}
```

### 5. Update Matcher (Optional)

If your backend has variant packages (like AUR's `-git`, `-bin` suffixes), update `src/core/matcher.rs`:

```rust
match target.backend {
    Backend::Aur => self.find_aur_package(target, installed_snapshot),
    Backend::Flatpak => self.find_flatpak_package(target, installed_snapshot),
    Backend::Soar => None,  // No variants
    Backend::Nala => self.find_nala_package(target, installed_snapshot),  // NEW
}
```

And in `is_same_package`:

```rust
match pkg1.backend {
    Backend::Aur => self.is_variant_match(pkg1, pkg2),
    Backend::Flatpak => {
        let name1 = pkg1.name.to_lowercase();
        let name2 = pkg2.name.to_lowercase();
        name1.contains(&name2) || name2.contains(&name1)
    }
    Backend::Soar => false,  // No variants
    Backend::Nala => false,  // NEW: No variants (exact match only)
}
```

### 6. Update Commands

Update `src/commands/sync.rs` and `src/commands/switch.rs` to handle the new backend in any match statements:

**sync.rs** (in smart matching sections):

```rust
match pkg.backend {
    Backend::Aur => { /* ... */ },
    Backend::Flatpak => { /* ... */ },
    Backend::Soar => {
        real_name = pkg.name.clone();  // No smart matching
    }
    Backend::Nala => {  // NEW
        real_name = pkg.name.clone();  // No smart matching
    }
}
```

**switch.rs** (in `determine_backend`):

```rust
fn determine_backend(package_name: &str, backend_opt: Option<String>) -> Result<Backend> {
    if let Some(backend_str) = backend_opt {
        match backend_str.to_lowercase().as_str() {
            "aur" => Ok(Backend::Aur),
            "flatpak" => Ok(Backend::Flatpak),
            "soar" => Ok(Backend::Soar),
            "nala" => Ok(Backend::Nala),  // NEW
            _ => Err(DeclarchError::Other(format!(
                "Unknown backend: {}. Use 'aur', 'flatpak', 'soar', or 'nala'",
                backend_str
            ))),
        }
    } else {
        // Auto-detect based on prefix
        if package_name.starts_with("flatpak:") {
            Ok(Backend::Flatpak)
        } else if package_name.starts_with("soar:") {
            Ok(Backend::Soar)
        } else if package_name.starts_with("nala:") {  // NEW
            Ok(Backend::Nala)
        } else {
            Ok(Backend::Aur)
        }
    }
}
```

**info.rs** (for display):

```rust
match pkg_state.backend {
    crate::state::types::Backend::Aur => {
        println!("  {} {}", "→".dimmed(), name);
    },
    crate::state::types::Backend::Flatpak => {
        println!("  {} {} {}", "flt".green(), "→".dimmed(), name);
    },
    crate::state::types::Backend::Soar => {
        println!("  {} {} {}", "soar".blue(), "→".dimmed(), name);
    },
    crate::state::types::Backend::Nala => {  // NEW
        println!("  {} {} {}", "nala".cyan(), "→".dimmed(), name);
    },
}
```

### 7. Update Distro Detection (if needed)

If your backend is distro-specific, update `src/utils/distro.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistroType {
    Arch,
    Debian,
    Fedora,
    Unknown,
}

impl DistroType {
    pub fn supports_aur(&self) -> bool {
        matches!(self, DistroType::Arch)
    }

    pub fn supports_nala(&self) -> bool {  // NEW
        matches!(self, DistroType::Debian)
    }
}
```

### 8. Add Tests

Add tests to `src/packages/nala.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nala_manager_creation() {
        let manager = NalaManager::new(false);
        assert_eq!(manager.backend_type(), Backend::Nala);
        assert!(!manager.noconfirm);
    }

    #[test]
    fn test_nala_manager_noconfirm() {
        let manager = NalaManager::new(true);
        assert!(manager.noconfirm);
    }
}
```

## Example Configuration

After adding the Nala backend, users can configure it like this:

```kdl
// Cross-distro packages.declarch configuration

// Soar packages (works everywhere)
packages {
    bat
    exa
    ripgrep
}

// AUR packages (Arch-only)
aur-packages {
    hyprland
    waybar
}

// Nala packages (Debian/Ubuntu-only)
nala-packages {
    neovim
    ffmpeg
    build-essential
}

// Flatpak packages (cross-distro)
flatpak-packages {
    com.spotify.Client
    org.mozilla.firefox
}
```

## Backend Availability

- **Arch Linux**: AUR + Soar + Flatpak
- **Debian/Ubuntu**: Nala + Soar + Flatpak (AUR packages are silently skipped)
- **Fedora**: Soar + Flatpak (AUR and Nala packages are skipped)
- **Unknown**: Soar + Flatpak (fallback)

## Testing Your Backend

```bash
# Run unit tests
cargo test

# Build with debug output
cargo build

# Test with dry-run
declarch sync --dry-run

# Test specific backend
declarch sync nala  # or aur, flatpak, soar
```

## Best Practices

1. **Error Handling**: Always return proper `Result` types with descriptive errors
2. **Availability Check**: Implement `is_available()` to check if the package manager exists
3. **Smart Matching**: Only implement if your backend has package variants
4. **Distro Awareness**: Consider if your backend should be available on all distros
5. **Testing**: Add comprehensive tests for all methods
6. **Documentation**: Document any backend-specific quirks or requirements

## Summary Checklist

- [ ] Add `Backend::Name` to enum in `src/core/types.rs`
- [ ] Update `Display`, `FromStr` for `PackageId` and `Backend`
- [ ] Create `src/packages/name.rs` implementing `PackageManager`
- [ ] Add to `src/packages/mod.rs`
- [ ] Register in `src/packages/registry.rs`
- [ ] Update `src/config/kdl.rs` parser
- [ ] Update `src/config/loader.rs` to load packages
- [ ] Update matcher (if variants exist)
- [ ] Update `sync.rs`, `switch.rs`, `info.rs`
- [ ] Update distro detection (if needed)
- [ ] Add tests
- [ ] Build and test

That's it! You've successfully added a new package manager backend to declarch.
