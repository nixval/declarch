use crate::config::loader::{self, MergedConfig};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use std::collections::HashMap;
use std::path::Path;

use super::matching::load_known_backends;
use super::query_helpers::resolve_module_path;

pub(super) fn explain_target(target: &str, config: &MergedConfig) -> Result<()> {
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
        "sync-plan" => super::sync_plan::explain_sync_plan(config),
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

pub(super) fn explain_module(
    query: &str,
    module_path: &Path,
    root_config: &MergedConfig,
) -> Result<()> {
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

pub(super) fn explain_backend(
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
