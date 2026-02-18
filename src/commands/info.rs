use crate::error::Result;
use crate::state;
use crate::ui as output;

mod diagnostics;
mod output_view;

pub struct InfoOptions {
    pub doctor: bool,
    pub format: Option<String>,
    pub output_version: Option<String>,
    pub backend: Option<String>,
    pub package: Option<String>,
    pub verbose: bool,
}

pub fn run(options: InfoOptions) -> Result<()> {
    if options.doctor {
        return run_doctor(options.verbose);
    }

    let result = run_info(&options);
    if result.is_ok() && !matches!(options.format.as_deref(), Some("json" | "yaml")) {
        maybe_print_update_notification();
    }
    result
}

fn run_info(options: &InfoOptions) -> Result<()> {
    let state = state::io::load_state()?;

    let filtered_packages: Vec<(&String, &state::types::PackageState)> =
        if options.backend.is_some() || options.package.is_some() {
            let backend_filter = options.backend.as_deref();
            let package_filter = options.package.as_deref();

            state
                .packages
                .iter()
                .filter(|(key, pkg_state)| {
                    let name = extract_package_name(key);

                    if let Some(filter_pkg) = package_filter
                        && !name.contains(filter_pkg)
                    {
                        return false;
                    }

                    if let Some(filter_backend) = backend_filter {
                        pkg_state.backend == crate::core::types::Backend::from(filter_backend)
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            state.packages.iter().collect()
        };

    let format_str = options.format.as_deref().unwrap_or("table");
    if options.verbose && !matches!(format_str, "json" | "yaml") {
        output::header("Info Context");
        output::keyval(
            "Backend filter",
            options.backend.as_deref().unwrap_or("(none)"),
        );
        output::keyval(
            "Package filter",
            options.package.as_deref().unwrap_or("(none)"),
        );
        output::keyval(
            "State file",
            &state::io::get_state_path()?.display().to_string(),
        );
    }

    match format_str {
        "json" => output_json_filtered(&filtered_packages, options.output_version.as_deref()),
        "yaml" => output_yaml_filtered(&filtered_packages, options.output_version.as_deref()),
        _ => output_table_filtered(&state, &filtered_packages),
    }
}

fn output_table_filtered(
    state: &state::types::State,
    filtered_packages: &[(&String, &state::types::PackageState)],
) -> Result<()> {
    output_view::output_table_filtered(state, filtered_packages)
}

fn output_json_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    output_view::output_json_filtered(filtered_packages, output_version)
}

fn output_yaml_filtered(
    filtered_packages: &[(&String, &state::types::PackageState)],
    output_version: Option<&str>,
) -> Result<()> {
    output_view::output_yaml_filtered(filtered_packages, output_version)
}

fn extract_package_name(key: &str) -> &str {
    key.split_once(':').map(|(_, n)| n).unwrap_or(key)
}

fn run_doctor(verbose: bool) -> Result<()> {
    diagnostics::run_doctor(verbose)
}

fn maybe_print_update_notification() {
    diagnostics::maybe_print_update_notification()
}

#[cfg(test)]
mod tests;
