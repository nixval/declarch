use crate::config::loader;
use crate::error::{DeclarchError, Result};
use crate::ui as output;
use crate::utils::paths;
use std::path::{Path, PathBuf};

pub(super) fn load_single_module(
    _config_path: &Path,
    module_name: &str,
    selectors: &loader::LoadSelectors,
) -> Result<loader::MergedConfig> {
    let module_path = paths::module_file(module_name);

    let final_path = if let Ok(path) = module_path {
        if path.exists() {
            path
        } else {
            let direct_path = PathBuf::from(module_name);
            if direct_path.exists() {
                direct_path
            } else {
                return Err(DeclarchError::Other(format!(
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
            return Err(DeclarchError::Other(format!(
                "Module not found: {}",
                module_name
            )));
        }
    };

    let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
    Ok(module_config)
}

pub(super) fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
    selectors: &loader::LoadSelectors,
    verbose: bool,
) -> Result<loader::MergedConfig> {
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
                    return Err(DeclarchError::Other(format!(
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
                return Err(DeclarchError::Other(format!(
                    "Module not found: {}",
                    module_name
                )));
            }
        };

        if verbose {
            output::verbose(&format!("Loading module: {}", final_path.display()));
        }
        let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}
