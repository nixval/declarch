use crate::config::loader::MergedConfig;
use crate::state::types::{State, Backend};
use crate::utils::errors::Result;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub to_install: Vec<(String, Backend)>,
    pub to_prune: Vec<(String, Backend)>,
    pub to_adopt: Vec<(String, Backend)>,
}

pub trait SystemInspector {
    fn is_installed(&self, pkg: &str, backend: &Backend) -> Result<bool>;
}

pub fn resolve(
    config: &MergedConfig, 
    state: &State, 
    inspector: &impl SystemInspector
) -> Result<Transaction> {
    let mut tx = Transaction {
        to_install: vec![],
        to_prune: vec![],
        to_adopt: vec![],
    };

    let mut config_map: HashMap<String, Backend> = HashMap::new();
    
    for pkg_str in &config.packages {
        let (name, backend) = parse_package_string(pkg_str);
        if !config.excludes.contains(&name) {
            config_map.insert(name, backend);
        }
    }

    for (name, pkg_state) in &state.packages {
        if !config_map.contains_key(name) {
            tx.to_prune.push((name.clone(), pkg_state.backend.clone()));
        }
    }

    for (name, backend) in config_map {
        if state.packages.contains_key(&name) {
            continue;
        }

        let is_installed = inspector.is_installed(&name, &backend)?;

        if is_installed {
            tx.to_adopt.push((name, backend));
        } else {
            tx.to_install.push((name, backend));
        }
    }

    Ok(tx)
}

fn parse_package_string(pkg: &str) -> (String, Backend) {
    if let Some(stripped) = pkg.strip_prefix("flatpak:") {
        (stripped.to_string(), Backend::Flatpak)
    } else {
        (pkg.to_string(), Backend::Aur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::PackageState;
    use chrono::Utc;

    struct MockInspector {
        installed: Vec<String>,
    }

    impl SystemInspector for MockInspector {
        fn is_installed(&self, pkg: &str, _backend: &Backend) -> Result<bool> {
            Ok(self.installed.contains(&pkg.to_string()))
        }
    }

    #[test]
    fn test_install_new_package() {
        let config = MergedConfig {
            packages: vec!["firefox".to_string()],
            excludes: vec![],
        };
        let state = State::default();
        let inspector = MockInspector { installed: vec![] };

        let tx = resolve(&config, &state, &inspector).unwrap();
        
        assert_eq!(tx.to_install.len(), 1);
        assert_eq!(tx.to_install[0].0, "firefox");
        assert!(tx.to_adopt.is_empty());
    }

    #[test]
    fn test_adopt_existing_package() {
        let config = MergedConfig {
            packages: vec!["vim".to_string()],
            excludes: vec![],
        };
        let state = State::default();
        let inspector = MockInspector { installed: vec!["vim".to_string()] };

        let tx = resolve(&config, &state, &inspector).unwrap();
        
        assert!(tx.to_install.is_empty());
        assert_eq!(tx.to_adopt.len(), 1);
        assert_eq!(tx.to_adopt[0].0, "vim");
    }

    #[test]
    fn test_prune_removed_package() {
        let config = MergedConfig::default(); 
        let mut state = State::default();
        state.packages.insert("htop".to_string(), PackageState {
            backend: Backend::Aur,
            installed_at: Utc::now(),
            version: None,
        });
        
        let inspector = MockInspector { installed: vec!["htop".to_string()] };

        let tx = resolve(&config, &state, &inspector).unwrap();

        assert_eq!(tx.to_prune.len(), 1);
        assert_eq!(tx.to_prune[0].0, "htop");
    }

    #[test]
    fn test_exclude_prevents_install() {
        let config = MergedConfig {
            packages: vec!["bloatware".to_string()],
            excludes: vec!["bloatware".to_string()],
        };
        let state = State::default();
        let inspector = MockInspector { installed: vec![] };

        let tx = resolve(&config, &state, &inspector).unwrap();
        
        assert!(tx.to_install.is_empty());
    }
}
