use crate::config::editor;
use crate::error::{DeclarchError, Result};
use crate::utils::sanitize::validate_package_name;

#[derive(Debug, Clone)]
pub(super) struct PlannedInstall {
    pub package: String,
    pub backend: String,
}

pub(super) fn plan_installs(
    raw_packages: &[String],
    backend_flag: Option<&String>,
) -> Result<Vec<PlannedInstall>> {
    let mut planned = Vec::with_capacity(raw_packages.len());

    for raw in raw_packages {
        let (backend_override, pkg_name) = editor::parse_package_string(raw)?;
        validate_package_name(&pkg_name)?;
        let backend = backend_override
            .or_else(|| backend_flag.cloned())
            .ok_or_else(|| {
                DeclarchError::Other(format!(
                    "Package '{}' has no backend. Use '<backend>:{}' or '--backend <name>'.",
                    pkg_name, pkg_name
                ))
            })?;

        planned.push(PlannedInstall {
            package: pkg_name,
            backend,
        });
    }

    Ok(planned)
}
