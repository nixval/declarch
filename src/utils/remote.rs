mod security;
mod url_builders;

use crate::constants::{CONFIG_EXTENSION, PROJECT_NAME};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use reqwest::blocking::Client;
use security::validate_url;
#[cfg(test)]
use security::{first_private_ip, is_private_address};
#[cfg(test)]
use std::net::IpAddr;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use url_builders::{build_backend_urls, build_urls};

static DEFAULT_REGISTRY: LazyLock<String> = LazyLock::new(project_identity::registry_raw_base_url);
static BACKENDS_REGISTRY: LazyLock<String> =
    LazyLock::new(|| format!("{}/backends", *DEFAULT_REGISTRY));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Fetch module content from remote repository
///
/// Supports multiple sources:
///
/// 1. **Default registry** (declarch-packages):
///    ```bash
///    declarch init hyprland/niri-nico
///    # → https://raw.githubusercontent.com/nixval/declarch-packages/main/modules/hyprland/niri-nico.kdl
///    ```
///
/// 2. **Arbitrary GitHub repo** (Go-style):
///    ```bash
///    declarch init myuser/hyprland1
///    # → https://raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl
///    declarch init hyprwm/hyprland
///    # → https://raw.githubusercontent.com/hyprwm/hyprland/main/declarch.kdl
///    ```
///
/// 3. **GitLab repo**:
///    ```bash
///    declarch init gitlab.com/user/repo
///    # → https://gitlab.com/user/repo/-/raw/main/declarch.kdl
///    ```
///
/// 4. **Direct URL**:
///    ```bash
///    declarch init https://example.com/config.kdl
///    ```
pub fn fetch_module_content(target_path: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| DeclarchError::RemoteFetchError(format!("HTTP client creation: {}", e)))?;

    // Try different URL patterns
    let urls = build_urls(target_path);

    let mut failures = Vec::new();
    for url in urls {
        match fetch_url(&client, &url) {
            Ok(content) => {
                // Show successful fetch
                output::info(&format!("fetch: {}", url));

                // Add header comment
                let mut final_content = String::new();
                final_content.push_str(&format!("// Source: {}\n", target_path));
                final_content.push_str(&format!("// URL: {}\n", url));
                final_content.push('\n');
                final_content.push_str(&content);

                return Ok(final_content);
            }
            Err(e) => failures.push(format_fetch_failure(&url, &e.to_string())),
        }
    }

    Err(DeclarchError::TargetNotFound(format!(
        "Failed to fetch from: {}\n  Hint: Ensure the repository has a {}.{} file\n{}",
        target_path,
        PROJECT_NAME,
        CONFIG_EXTENSION,
        format_failure_summary(&failures)
    )))
}

/// Fetch backend configuration from remote repository
///
/// Tries to fetch from declarch-packages/backends/ first,
/// then falls back to local template generation if not found.
pub fn fetch_backend_content(backend_name: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| DeclarchError::Other(format!("Failed to create HTTP client: {}", e)))?;

    // Build URLs to try for backend
    let urls = build_backend_urls(backend_name);

    let mut failures = Vec::new();
    for url in urls {
        let mut last_error: Option<DeclarchError> = None;

        // Retry transient backend fetch failures once. This reduces flakiness
        // when GitHub raw endpoints or DNS have short hiccups.
        for attempt in 1..=2 {
            match fetch_url(&client, &url) {
                Ok(content) => {
                    output::info(&format!("fetch backend: {}", url));
                    return Ok(content);
                }
                Err(e) => {
                    let should_retry = attempt == 1 && is_retryable_fetch_error(&e);
                    last_error = Some(e);
                    if should_retry {
                        thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                    break;
                }
            }
        }

        if let Some(err) = last_error {
            failures.push(format_fetch_failure(&url, &err.to_string()));
        }
    }

    Err(DeclarchError::TargetNotFound(format!(
        "Backend '{}' not found in registry.\n{}",
        backend_name,
        format_failure_summary(&failures)
    )))
}

fn is_retryable_fetch_error(err: &DeclarchError) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    !(msg.contains("http 404") || msg.contains("not found"))
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
    // Validate URL scheme before making request
    validate_url(url)?;

    let resp = client
        .get(url)
        .header(
            "User-Agent",
            format!("{}-cli", project_identity::BINARY_NAME),
        )
        .send()
        .map_err(|e| DeclarchError::RemoteFetchError(format!("Network request: {}", e)))?;

    if resp.status().is_success() {
        resp.text()
            .map_err(|e| DeclarchError::RemoteFetchError(format!("Response read: {}", e)))
    } else {
        Err(DeclarchError::RemoteFetchError(format!(
            "HTTP {}",
            resp.status()
        )))
    }
}

fn format_fetch_failure(url: &str, reason: &str) -> String {
    format!("- {} => {}", url, reason)
}

fn format_failure_summary(failures: &[String]) -> String {
    if failures.is_empty() {
        return "No URL candidates were generated.".to_string();
    }

    let mut out = String::from("Fetch attempts:");
    for line in failures.iter().take(3) {
        out.push('\n');
        out.push_str("  ");
        out.push_str(line);
    }
    if failures.len() > 3 {
        out.push_str(&format!("\n  ... and {} more", failures.len() - 3));
    }
    out
}

#[cfg(test)]
mod tests;
