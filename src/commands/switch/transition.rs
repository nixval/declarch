use crate::core::types::Backend;
use crate::error::{DeclarchError, Result};
use crate::packages::PackageManager;
use crate::project_identity;
use crate::ui as output;
use colored::Colorize;

/// Determine backend and strip optional backend prefixes from package names.
/// Returns `(backend, old_package_name, new_package_name)`.
pub(super) fn determine_target(
    old_package: &str,
    new_package: &str,
    backend_opt: Option<&str>,
) -> Result<(Backend, String, String)> {
    fn extract_backend(name: &str) -> Option<Backend> {
        name.split_once(':').map(|(b, _)| Backend::from(b))
    }

    fn strip_prefix(name: &str) -> &str {
        name.split_once(':').map(|(_, n)| n).unwrap_or(name)
    }

    if let Some(backend_str) = backend_opt {
        let backend = Backend::from(backend_str);
        return Ok((
            backend,
            strip_prefix(old_package).to_string(),
            strip_prefix(new_package).to_string(),
        ));
    }

    let old_backend = extract_backend(old_package);
    let new_backend = extract_backend(new_package);

    match (old_backend, new_backend) {
        (Some(old), Some(new)) => {
            if old != new {
                return Err(DeclarchError::Other(
                    "Cross-backend switch is not supported. Both packages must use the same backend."
                        .to_string(),
                ));
            }
            Ok((
                old,
                strip_prefix(old_package).to_string(),
                strip_prefix(new_package).to_string(),
            ))
        }
        (Some(old), None) => Ok((
            old,
            strip_prefix(old_package).to_string(),
            strip_prefix(new_package).to_string(),
        )),
        (None, Some(new)) => Ok((
            new,
            strip_prefix(old_package).to_string(),
            strip_prefix(new_package).to_string(),
        )),
        (None, None) => {
            let registry = crate::packages::get_registry();
            let backends = registry
                .lock()
                .map(|r| r.available_backends())
                .unwrap_or_default();

            let backend_list = if backends.is_empty() {
                format!(
                    "No backends configured. Run '{}' first.",
                    project_identity::cli_with("init")
                )
            } else {
                format!("Available backends: {}", backends.join(", "))
            };

            Err(DeclarchError::Other(format!(
                "Cannot determine backend.\n\n\
                 Use explicit prefix syntax:\n\
                   {} {}:{} {}:{}\n\n\
                 Or specify backend:\n\
                   {} {} {} --backend <BACKEND>\n\n\
                 {}",
                project_identity::BINARY_NAME,
                backends.first().unwrap_or(&"BACKEND".to_string()),
                strip_prefix(old_package),
                backends.first().unwrap_or(&"BACKEND".to_string()),
                strip_prefix(new_package),
                project_identity::BINARY_NAME,
                strip_prefix(old_package),
                strip_prefix(new_package),
                backend_list
            )))
        }
    }
}

pub(super) fn execute_transition(
    old_package: &str,
    new_package: &str,
    _backend: &Backend,
    manager: &dyn PackageManager,
) -> Result<()> {
    output::indent(&format!("Uninstalling {}...", old_package.yellow()), 0);
    manager
        .remove(&[old_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to uninstall {}: {}", old_package, e)))?;

    output::indent(&format!("Installing {}...", new_package.green()), 0);
    manager
        .install(&[new_package.to_string()])
        .map_err(|e| DeclarchError::Other(format!("Failed to install {}: {}", new_package, e)))?;

    Ok(())
}
