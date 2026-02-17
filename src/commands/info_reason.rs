use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::core::resolver;
use crate::core::types::{Backend, PackageId};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

mod query_helpers;
use query_helpers::{
    load_config_with_modules, looks_like_module_query, parse_query, resolve_module_path,
};

pub struct InfoReasonOptions {
    pub query: Option<String>,
    pub target: Option<String>,
    pub profile: Option<String>,
    pub host: Option<String>,
    pub modules: Vec<String>,
    pub verbose: bool,
}

pub fn run(options: InfoReasonOptions) -> Result<()> {
    if options.query.is_none() && options.target.is_none() {
        return Err(DeclarchError::Other(
            "Nothing to inspect. Provide a QUERY or use --plan.".to_string(),
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

    if options.verbose {
        output::header("Verbose Context");
        output::keyval("Config file", &config_path.display().to_string());
        output::keyval("Declared packages", &config.packages.len().to_string());
        output::keyval(
            "Known backends",
            &load_known_backends(&config).len().to_string(),
        );
    }

    if let Some(target) = options.target {
        return explain_target(&target, &config);
    }

    let query = options.query.unwrap_or_default();
    explain_query(&query, &config)
}

fn explain_target(target: &str, config: &MergedConfig) -> Result<()> {
    let known_backends = load_known_backends(config);

    match target {
        "help" | "targets" => {
            output::header("Info Targets");
            output::info("Use one of these target forms:");
            output::indent("• --target sync-plan", 1);
            output::indent("• --target backend:<name>", 1);
            output::indent("• --target module:<path>", 1);
            output::separator();
            output::info("Examples:");
            output::indent(
                &format!("• {}", project_identity::cli_with("info --plan")),
                1,
            );
            output::indent(
                &format!("• {}", project_identity::cli_with("info backend:pnpm")),
                1,
            );
            output::indent(
                &format!("• {}", project_identity::cli_with("info system/base")),
                1,
            );
            Ok(())
        }
        "sync-plan" => explain_sync_plan(config),
        t if t.starts_with("backend:") => {
            let backend_name = t.trim_start_matches("backend:").trim();
            if backend_name.is_empty() {
                return Err(DeclarchError::Other(
                    "Backend target is empty. Use --target backend:<name>".to_string(),
                ));
            }
            explain_backend(backend_name, config, &known_backends)
        }
        t if t.starts_with("module:") => {
            let module_name = t.trim_start_matches("module:").trim();
            if module_name.is_empty() {
                return Err(DeclarchError::Other(
                    "Module target is empty. Use --target module:<name>".to_string(),
                ));
            }
            let module_path = resolve_module_path(module_name)?;
            explain_module(module_name, &module_path, config)
        }
        other => Err(DeclarchError::Other(format!(
            "Unsupported info target '{}'. Supported: sync-plan, backend:<name>, module:<name>.",
            other
        ))),
    }
}

fn explain_sync_plan(config: &MergedConfig) -> Result<()> {
    output::header("Info: plan");

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
    let known_backends = load_known_backends(config);

    output::header(&format!("Info: {}", query));

    // Module intent: explain module content/source when query looks like module/path.
    if looks_like_module_query(&needle)
        && let Ok(module_path) = resolve_module_path(&needle)
    {
        return explain_module(&needle, &module_path, config);
    }

    // Backend intent: "backend:name" syntax, or plain backend name.
    if backend_filter
        .as_ref()
        .is_some_and(|b| b.name() == "backend")
    {
        return explain_backend(&needle, config, &known_backends);
    }

    let mut exact_matches = find_matches(config, backend_filter.as_ref(), &needle, true);
    if exact_matches.is_empty() {
        exact_matches = find_matches(config, backend_filter.as_ref(), &needle, false);
    }

    if exact_matches.is_empty() {
        if known_backends.contains_key(&needle.to_lowercase()) {
            return explain_backend(&needle, config, &known_backends);
        }

        output::warning(&format!("Nothing matched '{}'.", query));
        output::info("Try one of these:");
        output::indent(
            &format!("• {}", project_identity::cli_with("info <package-name>")),
            1,
        );
        output::indent(
            &format!(
                "• {}",
                project_identity::cli_with("info <backend>:<package>")
            ),
            1,
        );
        output::indent(
            &format!(
                "• {}",
                project_identity::cli_with("info backend:<backend-name>")
            ),
            1,
        );
        output::indent(
            &format!("• {}", project_identity::cli_with("info <module-name>")),
            1,
        );
        output::indent(
            &format!("• {}", project_identity::cli_with("info --plan")),
            1,
        );
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

fn explain_module(query: &str, module_path: &Path, root_config: &MergedConfig) -> Result<()> {
    output::separator();
    output::keyval("Kind", "module");
    output::keyval("Module query", query);
    output::keyval("Resolved path", &module_path.display().to_string());

    let module_config = loader::load_root_config(module_path)?;
    output::keyval(
        "Declared packages",
        &module_config.packages.len().to_string(),
    );

    if module_config.packages.is_empty() {
        output::info("This module currently has no package entries.");
        return Ok(());
    }

    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in module_config.packages.keys() {
        grouped
            .entry(pkg.backend.to_string())
            .or_default()
            .push(pkg.name.clone());
    }

    let mut backends: Vec<_> = grouped.keys().cloned().collect();
    backends.sort();

    output::info("Packages by backend:");
    for backend in backends {
        let mut pkgs = grouped.remove(&backend).unwrap_or_default();
        pkgs.sort();
        println!("  {}: {}", backend, pkgs.join(", "));
    }

    let root_sources = root_config
        .packages
        .values()
        .flatten()
        .filter(|p| p == &module_path)
        .count();
    output::keyval("Referenced in merged config", &root_sources.to_string());

    Ok(())
}

fn explain_backend(
    backend_name: &str,
    config: &MergedConfig,
    known_backends: &HashMap<String, crate::backends::config::BackendConfig>,
) -> Result<()> {
    let key = backend_name.to_lowercase();
    let Some(backend) = known_backends.get(&key) else {
        return Err(DeclarchError::TargetNotFound(format!(
            "backend:{}",
            backend_name
        )));
    };

    output::separator();
    output::keyval("Kind", "backend");
    output::keyval("Backend", &backend.name);
    output::keyval("Binary", &backend.binary.primary());
    output::keyval(
        "Supported OS",
        &crate::utils::platform::supported_os_summary(backend),
    );
    output::keyval("Current OS", crate::utils::platform::current_os_tag());

    if crate::utils::platform::backend_supports_current_os(backend) {
        output::success("Backend is compatible on this device");
    } else {
        output::warning("Backend is not for this OS, so related actions will be skipped");
    }

    let referenced_count = config
        .packages
        .keys()
        .filter(|pkg| pkg.backend.name() == backend.name.to_lowercase())
        .count();
    output::keyval("Packages using backend", &referenced_count.to_string());

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

    matches.sort_by_key(|a| a.to_string());
    matches
}

fn load_known_backends(
    config: &MergedConfig,
) -> HashMap<String, crate::backends::config::BackendConfig> {
    let mut backends = crate::backends::load_all_backends_unified().unwrap_or_default();
    for backend in &config.backends {
        backends.insert(backend.name.to_lowercase(), backend.clone());
    }
    backends
}

fn show_active_context(options: &InfoReasonOptions, config: &MergedConfig) {
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

#[cfg(test)]
mod tests;
