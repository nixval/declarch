mod url_builders;

use crate::constants::{CONFIG_EXTENSION, PROJECT_NAME};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use reqwest::blocking::Client;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;
use url_builders::{build_backend_urls, build_urls};

static DEFAULT_REGISTRY: LazyLock<String> = LazyLock::new(project_identity::registry_raw_base_url);
static BACKENDS_REGISTRY: LazyLock<String> =
    LazyLock::new(|| format!("{}/backends", *DEFAULT_REGISTRY));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Secure scheme allowed by default.
const SECURE_SCHEME: &str = "https";
/// Optional insecure scheme. Only allowed with explicit opt-in.
const INSECURE_SCHEME: &str = "http";

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

/// Validate URL to prevent SSRF attacks
fn validate_url(url_str: &str) -> Result<()> {
    // Parse URL to validate scheme
    let parsed = reqwest::Url::parse(url_str)
        .map_err(|_| DeclarchError::RemoteFetchError(format!("Invalid URL: {}", url_str)))?;

    // Check scheme policy (HTTPS by default, HTTP opt-in only).
    let scheme = parsed.scheme();
    if !is_allowed_scheme(scheme) {
        let insecure_key = project_identity::env_key("ALLOW_INSECURE_HTTP");
        return Err(DeclarchError::RemoteFetchError(format!(
            "URL scheme '{}' is blocked. Allowed by default: https. To allow http explicitly set {}=1.",
            scheme, insecure_key
        )));
    }

    // Prevent access to localhost/private networks (basic check)
    let host = parsed.host_str().ok_or_else(|| {
        DeclarchError::RemoteFetchError(format!("URL must include a valid host: {}", url_str))
    })?;
    if is_private_address(host) {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Access to private addresses is not allowed: {}",
            host
        )));
    }

    // Resolve hostname to prevent DNS-based SSRF bypasses where a public-looking
    // hostname resolves to private/local addresses.
    let port = parsed.port_or_known_default().unwrap_or(443);
    let resolved = resolve_host_addresses(host, port).map_err(|e| {
        DeclarchError::RemoteFetchError(format!("Failed to resolve host '{}': {}", host, e))
    })?;
    if resolved.is_empty() {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Failed to resolve host '{}': no addresses returned",
            host
        )));
    }
    if let Some(private_ip) = first_private_ip(&resolved) {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Access to private addresses is not allowed: {} -> {}",
            host, private_ip
        )));
    }

    Ok(())
}

fn is_allowed_scheme(scheme: &str) -> bool {
    if scheme == SECURE_SCHEME {
        return true;
    }

    if scheme == INSECURE_SCHEME {
        return project_identity::env_get("ALLOW_INSECURE_HTTP").unwrap_or_default() == "1";
    }

    false
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

fn resolve_host_addresses(host: &str, port: u16) -> std::io::Result<Vec<IpAddr>> {
    (host, port)
        .to_socket_addrs()
        .map(|iter| iter.map(|sa| sa.ip()).collect())
}

fn first_private_ip(addrs: &[IpAddr]) -> Option<IpAddr> {
    addrs.iter().copied().find(|ip| is_private_ip(*ip))
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local() || ipv4.is_unspecified()
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback()
                || ipv6.is_unique_local()
                || ipv6.is_unicast_link_local()
                || ipv6.is_unspecified()
        }
    }
}

/// Check if hostname is a private/local address
fn is_private_address(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(ip);
    }

    // Check for localhost variants
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests;
