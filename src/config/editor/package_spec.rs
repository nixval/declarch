use crate::error::{DeclarchError, Result};

/// Parse package string with optional backend
///
/// # Arguments
/// * `input` - Package string, e.g., "vim" or "soar:bat"
///
/// # Returns
/// * `(Option<backend>, package_name)` - Backend if specified, and package name
///
/// # Examples
/// ```
/// # use declarch::config::editor::parse_package_string;
/// # use declarch::error::Result;
/// # fn main() -> Result<()> {
/// assert_eq!(parse_package_string("vim")?, (None, "vim".to_string()));
/// assert_eq!(parse_package_string("soar:bat")?, (Some("soar".to_string()), "bat".to_string()));
/// assert_eq!(parse_package_string("npm:nodejs")?, (Some("npm".to_string()), "nodejs".to_string()));
/// # Ok(())
/// # }
/// ```
pub fn parse_package_string(input: &str) -> Result<(Option<String>, String)> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(DeclarchError::Other(
            "Package name cannot be empty".to_string(),
        ));
    }

    if trimmed.matches(':').count() > 1 {
        return Err(DeclarchError::Other(format!(
            "Invalid package format '{}'. Use 'backend:package' or 'package'",
            input
        )));
    }

    if let Some((backend, package)) = trimmed.split_once(':') {
        let backend = backend.trim();
        let package = package.trim();

        if backend.is_empty() {
            return Err(DeclarchError::Other(
                "Backend cannot be empty (use 'package' without colon)".to_string(),
            ));
        }

        if package.is_empty() {
            return Err(DeclarchError::Other(
                "Package name cannot be empty (use 'backend:' without package)".to_string(),
            ));
        }

        if is_valid_backend(backend) {
            Ok((Some(backend.to_string()), package.to_string()))
        } else {
            Err(DeclarchError::Other(format!(
                "Invalid backend name: '{}'. Backend names must contain only letters, numbers, and hyphens",
                backend
            )))
        }
    } else {
        Ok((None, trimmed.to_string()))
    }
}

pub(super) fn is_valid_backend(backend: &str) -> bool {
    !backend.is_empty()
        && backend
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}
