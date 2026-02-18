use crate::config::loader::{self, LoadSelectors, MergedConfig};
use crate::core::resolver;
use crate::core::types::PackageId;
use crate::error::{DeclarchError, Result};
use crate::state;
use crate::ui as output;
use crate::utils::paths;
use std::collections::BTreeSet;

mod matching;
mod presentation;
mod query_helpers;
mod sync_plan;
mod targets;
use matching::{find_matches, load_known_backends};
use presentation::{
    render_backend_missing_suggestions, render_context_verbose, show_active_context,
};
use query_helpers::{
    load_config_with_modules, looks_like_module_query, parse_query, resolve_module_path,
};
use targets::{explain_backend, explain_module, explain_target};

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
        render_context_verbose(&config_path, &config);
    }

    if let Some(target) = options.target {
        return explain_target(&target, &config);
    }

    let query = options.query.unwrap_or_default();
    explain_query(&query, &config)
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
        render_backend_missing_suggestions();
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

#[cfg(test)]
mod tests;
