use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::core::resolver;
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub struct ExplainOptions {
    pub query: Option<String>,
    pub target: Option<String>,
    pub profile: Option<String>,
    pub host: Option<String>,
    pub modules: Vec<String>,
}

pub fn run(options: ExplainOptions) -> Result<()> {
    if options.query.is_none() && options.target.is_none() {
        return Err(DeclarchError::Other(
            "Nothing to explain. Provide a QUERY or --target sync-plan".to_string(),
        ));
    }

    let config_path = paths::config_file()?;
    if !config_path.exists() {
        return Err(DeclarchError::ConfigNotFound { path: config_path });
    }

    let selectors = LoadSelectors {
        profile: options.profile.clone(),
        host: options.host.clone(),
    };

    let config = if options.modules.is_empty() {
        loader::load_root_config_with_selectors(&config_path, &selectors)?
    } else {
        load_config_with_modules(&config_path, &options.modules, &selectors)?
    };

    show_active_context(&options, &config);

    if let Some(target) = options.target {
        return explain_target(&target, &config);
    }

    let query = options.query.unwrap_or_default();
    explain_query(&query, &config)
}

fn explain_target(target: &str, config: &MergedConfig) -> Result<()> {
    match target {
        "sync-plan" => explain_sync_plan(config),
        other => Err(DeclarchError::Other(format!(
            "Unsupported explain target '{}'. Supported: sync-plan",
            other
        ))),
    }
}

fn explain_sync_plan(config: &MergedConfig) -> Result<()> {
    output::header("Explain: sync-plan");

    let state = state::io::load_state()?;

    let config_set: HashSet<PackageId> = config.packages.keys().cloned().collect();
    let mut state_set: HashSet<PackageId> = HashSet::new();

    for pkg_state in state.packages.values() {
        state_set.insert(PackageId {
            backend: pkg_state.backend.clone(),
            name: pkg_state.config_name.clone(),
        });
    }

    let to_install: Vec<_> = config_set.difference(&state_set).cloned().collect();
    let to_remove: Vec<_> = state_set.difference(&config_set).cloned().collect();

    output::keyval("Declared packages", &config.packages.len().to_string());
    output::keyval("State packages", &state.packages.len().to_string());
    output::keyval("Planned install", &to_install.len().to_string());
    output::keyval("Planned remove", &to_remove.len().to_string());

    let duplicates = config.get_duplicates();
    if !duplicates.is_empty() {
        output::warning(&format!(
            "Found {} duplicate declarations across modules",
            duplicates.len()
        ));
    }

    let conflicts = config.get_cross_backend_conflicts();
    if !conflicts.is_empty() {
        output::warning(&format!(
            "Found {} cross-backend package name conflicts",
            conflicts.len()
        ));
    }

    if to_install.is_empty() && to_remove.is_empty() {
        output::success("No install/remove drift detected");
        return Ok(());
    }

    if !to_install.is_empty() {
        output::separator();
        output::info("Why these will be installed:");
        for pkg in to_install.iter().take(15) {
            println!("  + {}", pkg);
        }
        if to_install.len() > 15 {
            println!("  ... and {} more", to_install.len() - 15);
        }
    }

    if !to_remove.is_empty() {
        output::separator();
        output::info("Why these will be removed:");
        for pkg in to_remove.iter().take(15) {
            println!("  - {}", pkg);
        }
        if to_remove.len() > 15 {
            println!("  ... and {} more", to_remove.len() - 15);
        }
    }

    Ok(())
}

fn explain_query(query: &str, config: &MergedConfig) -> Result<()> {
    let (backend_filter, needle) = parse_query(query);

    output::header(&format!("Explain: {}", query));

    let mut exact_matches = find_matches(config, backend_filter.as_ref(), &needle, true);
    if exact_matches.is_empty() {
        exact_matches = find_matches(config, backend_filter.as_ref(), &needle, false);
    }

    if exact_matches.is_empty() {
        return Err(DeclarchError::TargetNotFound(query.to_string()));
    }

    let state = state::io::load_state()?;

    for pkg in exact_matches {
        output::separator();
        output::keyval("Package", &pkg.to_string());

        let state_key = resolver::make_state_key(&pkg);
        if let Some(pkg_state) = state.packages.get(&state_key) {
            output::keyval("Installed", "yes");
            if let Some(actual) = &pkg_state.actual_package_name {
                output::keyval("Installed as", actual);
            }
            if let Some(version) = &pkg_state.version {
                output::keyval("Version", version);
            }
        } else {
            output::keyval("Installed", "no");
        }

        if let Some(sources) = config.packages.get(&pkg) {
            let mut srcs: BTreeSet<String> = BTreeSet::new();
            for src in sources {
                srcs.insert(src.display().to_string());
            }
            output::keyval("Declared in", &srcs.len().to_string());
            for src in srcs {
                println!("  - {}", src);
            }
        }

        let same_name: BTreeSet<String> = config
            .packages
            .keys()
            .filter(|candidate| candidate.name == pkg.name)
            .map(|candidate| candidate.backend.to_string())
            .collect();

        if same_name.len() > 1 {
            output::warning(&format!(
                "Same package name exists in multiple backends: {}",
                same_name.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
    }

    Ok(())
}

fn find_matches(
    config: &MergedConfig,
    backend_filter: Option<&Backend>,
    needle: &str,
    exact: bool,
) -> Vec<PackageId> {
    let needle_lower = needle.to_lowercase();

    let mut matches: Vec<_> = config
        .packages
        .keys()
        .filter(|pkg| backend_filter.is_none_or(|b| pkg.backend == *b))
        .filter(|pkg| {
            if exact {
                pkg.name.eq_ignore_ascii_case(needle)
            } else {
                pkg.name.to_lowercase().contains(&needle_lower)
            }
        })
        .cloned()
        .collect();

    matches.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    matches
}

fn parse_query(query: &str) -> (Option<Backend>, String) {
    if let Some((backend, name)) = query.split_once(':') {
        let backend = backend.trim();
        let name = name.trim();
        if !backend.is_empty() && !name.is_empty() {
            return (Some(Backend::from(backend)), name.to_string());
        }
    }

    (None, query.trim().to_string())
}

fn show_active_context(options: &ExplainOptions, config: &MergedConfig) {
    output::header("Active Context");

    output::keyval(
        "Profile",
        options.profile.as_deref().unwrap_or("(default only)"),
    );
    output::keyval("Host", options.host.as_deref().unwrap_or("(default only)"));

    if options.modules.is_empty() {
        output::keyval("Extra modules", "none");
    } else {
        output::keyval("Extra modules", &options.modules.join(", "));
    }

    output::keyval(
        "Backends in config",
        &config.get_backends().len().to_string(),
    );
}

fn load_config_with_modules(
    config_path: &Path,
    extra_modules: &[String],
    selectors: &LoadSelectors,
) -> Result<MergedConfig> {
    let mut merged = loader::load_root_config_with_selectors(config_path, selectors)?;

    for module_name in extra_modules {
        let final_path = resolve_module_path(module_name)?;
        output::info(&format!("  Loading module: {}", final_path.display()));

        let module_config = loader::load_root_config_with_selectors(&final_path, selectors)?;
        merged.packages.extend(module_config.packages);
        merged.excludes.extend(module_config.excludes);
    }

    Ok(merged)
}

fn resolve_module_path(module_name: &str) -> Result<PathBuf> {
    let module_path = paths::module_file(module_name);

    if let Ok(path) = module_path
        && path.exists()
    {
        return Ok(path);
    }

    let direct = PathBuf::from(module_name);
    if direct.exists() {
        return Ok(direct);
    }

    Err(DeclarchError::Other(format!(
        "Module not found: {}",
        module_name
    )))
}

#[cfg(test)]
mod tests {
    use super::parse_query;

    #[test]
    fn parse_query_backend_prefix() {
        let (backend, name) = parse_query("aur:bat");
        assert_eq!(backend.expect("backend").to_string(), "aur");
        assert_eq!(name, "bat");
    }

    #[test]
    fn parse_query_plain_name() {
        let (backend, name) = parse_query("firefox");
        assert!(backend.is_none());
        assert_eq!(name, "firefox");
    }
}
