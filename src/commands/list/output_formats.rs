use crate::error::Result;
use crate::state;
use crate::utils::machine_output;

pub(super) fn output_json(
    packages: &[&state::types::PackageState],
    output_version: Option<&str>,
) -> Result<()> {
    if output_version == Some("v1") {
        return machine_output::emit_v1("info --list", packages, Vec::new(), Vec::new(), "json");
    }

    let json = serde_json::to_string_pretty(packages)?;
    println!("{}", json);
    Ok(())
}

pub(super) fn output_yaml(
    packages: &[&state::types::PackageState],
    output_version: Option<&str>,
) -> Result<()> {
    if output_version == Some("v1") {
        return machine_output::emit_v1("info --list", packages, Vec::new(), Vec::new(), "yaml");
    }

    let json_value = serde_json::to_value(packages)?;
    let yaml = serde_yml::to_string(&json_value)?;
    println!("{}", yaml);
    Ok(())
}
