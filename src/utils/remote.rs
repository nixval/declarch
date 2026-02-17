use crate::constants::{CONFIG_EXTENSION, DEFAULT_BRANCHES, PROJECT_NAME};
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;
use reqwest::blocking::Client;
use std::env;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

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

/// Build list of URLs to try for backend
fn build_backend_urls(backend_name: &str) -> Vec<String> {
    let mut urls = Vec::new();

    // 1. Direct URL (starts with http:// or https://)
    if backend_name.starts_with("http://") || backend_name.starts_with("https://") {
        urls.push(backend_name.to_string());
        return urls;
    }

    // Clean name (remove .kdl if present)
    let clean_name = backend_name
        .strip_suffix(&format!(".{}", CONFIG_EXTENSION))
        .unwrap_or(backend_name);

    // 2. Default registry backends
    urls.push(format!(
        "{}/{}.{}",
        *BACKENDS_REGISTRY, clean_name, CONFIG_EXTENSION
    ));

    // 3. Try GitHub repo (owner/repo pattern for custom backends)
    if clean_name.contains('/') {
        let parts: Vec<&str> = clean_name.split('/').collect();
        if parts.len() >= 2 {
            let (owner, repo) = (parts[0], parts[1]);
            urls.push(format!(
                "https://raw.githubusercontent.com/{}/{}/main/backends/{}.{}",
                owner,
                repo,
                parts.last().unwrap_or(&"backend"),
                CONFIG_EXTENSION
            ));
        }
    }

    urls
}

/// Build list of URLs to try for the given target
fn build_urls(target: &str) -> Vec<String> {
    let mut urls = Vec::new();

    // 1. Direct URL (starts with http:// or https://) - pass through as-is
    if target.starts_with("http://") || target.starts_with("https://") {
        urls.push(target.to_string());
        return urls;
    }

    // Strip .kdl extension if present (for cleaner URL building)
    let clean_target = target
        .strip_suffix(&format!(".{}", CONFIG_EXTENSION))
        .unwrap_or(target);

    // 2. Config variant syntax: user/repo:variant or user/repo/branch:variant
    // Examples:
    //   - myuser/dotfiles:uwsm       → main branch, uwsm variant
    //   - myuser/dotfiles/develop:uwsm → develop branch, uwsm variant
    if clean_target.contains(':') && !clean_target.starts_with("gitlab.com/") {
        let parts: Vec<&str> = clean_target.split(':').collect();

        if parts.len() == 2 {
            let (repo_part, variant) = (parts[0], parts[1]);

            if repo_part.contains('/') {
                let repo_parts: Vec<&str> = repo_part.split('/').collect();

                if repo_parts.len() == 2 {
                    // user/repo:variant pattern
                    let (owner, repo_name) = (repo_parts[0], repo_parts[1]);

                    // Try: user/repo:variant → user/repo/main/declarch-variant.kdl
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/main/{}-{}.{}",
                        owner, repo_name, PROJECT_NAME, variant, CONFIG_EXTENSION
                    ));

                    // Also try branches (main, master)
                    if variant != "main" && variant != "master" {
                        for b in DEFAULT_BRANCHES {
                            urls.push(format!(
                                "https://raw.githubusercontent.com/{}/{}/{}/{}-{}.{}",
                                owner, repo_name, b, PROJECT_NAME, variant, CONFIG_EXTENSION
                            ));
                        }
                    }

                    return urls;
                } else if repo_parts.len() >= 3 {
                    // user/repo/branch:variant pattern
                    let (owner, repo_name, branch) = (repo_parts[0], repo_parts[1], repo_parts[2]);

                    // Try: user/repo/branch:variant → user/repo/branch/declarch-variant.kdl
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}-{}.{}",
                        owner, repo_name, branch, PROJECT_NAME, variant, CONFIG_EXTENSION
                    ));

                    // Also try main/master as fallback
                    if branch != "main" && branch != "master" {
                        for b in DEFAULT_BRANCHES {
                            urls.push(format!(
                                "https://raw.githubusercontent.com/{}/{}/{}/{}-{}.{}",
                                owner, repo_name, b, PROJECT_NAME, variant, CONFIG_EXTENSION
                            ));
                        }
                    }

                    return urls;
                }
            }
        }
    }

    // 3. GitLab URL (gitlab.com/user/repo or gitlab.com/user/repo/branch)
    if clean_target.starts_with("gitlab.com/") {
        let path = clean_target.strip_prefix("gitlab.com/").unwrap();
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() >= 2 {
            // gitlab.com/user/repo → https://gitlab.com/user/repo/-/raw/main/declarch.kdl
            // gitlab.com/user/repo/branch → https://gitlab.com/user/repo/-/raw/branch/declarch.kdl
            let (owner, repo, branch) = if parts.len() >= 3 {
                (parts[0], parts[1], parts[2])
            } else {
                (parts[0], parts[1], "main")
            };

            urls.push(format!(
                "https://gitlab.com/{}/{}/-/raw/{}/{}.{}",
                owner, repo, branch, PROJECT_NAME, CONFIG_EXTENSION
            ));
        }

        return urls;
    }

    // 4. Registry module path (has .kdl extension, like "gaming/steam-setup.kdl")
    // This takes priority over GitHub user/repo to avoid ambiguity
    if target.contains('/') && target.ends_with(&format!(".{}", CONFIG_EXTENSION)) {
        // It's a registry module path with extension
        urls.push(format!(
            "{}/modules/{}",
            *DEFAULT_REGISTRY,
            target // Keep .kdl extension
        ));

        // Also try GitHub as fallback
        let parts: Vec<&str> = clean_target.split('/').collect();
        if parts.len() >= 2 {
            let (owner, repo) = (parts[0], parts[1]);
            urls.push(format!(
                "https://raw.githubusercontent.com/{}/{}/main/{}.{}",
                owner, repo, PROJECT_NAME, CONFIG_EXTENSION
            ));
        }

        return urls;
    }

    // 5. GitHub-style URLs (owner/repo or owner/repo/branch)
    if clean_target.contains('/') {
        let parts: Vec<&str> = clean_target.split('/').collect();

        if parts.len() >= 2 {
            // Try with explicit branch
            let (owner, repo, branch) = if parts.len() >= 3 {
                (parts[0], parts[1], parts[2])
            } else {
                (parts[0], parts[1], "main")
            };

            // Try common branches (deduplicate)
            let branches_to_try = if branch == "main" || branch == "master" {
                vec![branch]
            } else {
                let mut branches = vec![branch];
                branches.extend(DEFAULT_BRANCHES.iter().cloned());
                branches
            };

            for b in branches_to_try {
                urls.push(format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/{}.{}",
                    owner, repo, b, PROJECT_NAME, CONFIG_EXTENSION
                ));
            }

            // Also try modules/ path for default registry (without .kdl)
            urls.push(format!(
                "{}/modules/{}.{}",
                *DEFAULT_REGISTRY, clean_target, CONFIG_EXTENSION
            ));
        }

        return urls;
    }

    // 6. Flat name → try default registry
    urls.push(format!(
        "{}/modules/{}.{}",
        *DEFAULT_REGISTRY, clean_target, CONFIG_EXTENSION
    ));

    urls
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
    // Validate URL scheme before making request
    validate_url(url)?;

    let resp = client
        .get(url)
        .header("User-Agent", "declarch-cli")
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
        return Err(DeclarchError::RemoteFetchError(format!(
            "URL scheme '{}' is blocked. Allowed by default: https. To allow http explicitly set DECLARCH_ALLOW_INSECURE_HTTP=1.",
            scheme
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
        return env::var("DECLARCH_ALLOW_INSECURE_HTTP").unwrap_or_default() == "1";
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
mod tests {
    use super::*;

    #[test]
    fn test_build_urls_github_default() {
        let urls = build_urls("myuser/hyprland1");

        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_github_with_branch() {
        let urls = build_urls("myuser/hyprland1/develop");

        assert!(urls.iter().any(|u| {
            u.contains("raw.githubusercontent.com/myuser/hyprland1/develop/declarch.kdl")
        }));
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_gitlab() {
        let urls = build_urls("gitlab.com/user/repo");

        assert!(
            urls.iter()
                .any(|u| u.contains("gitlab.com/user/repo/-/raw/main/declarch.kdl"))
        );
    }

    #[test]
    fn test_build_urls_gitlab_with_branch() {
        let urls = build_urls("gitlab.com/user/repo/develop");

        assert!(
            urls.iter()
                .any(|u| u.contains("gitlab.com/user/repo/-/raw/develop/declarch.kdl"))
        );
    }

    #[test]
    fn test_build_urls_direct_url() {
        let urls = build_urls("https://example.com/config.kdl");

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/config.kdl");
    }

    #[test]
    fn test_build_urls_flat_name() {
        let urls = build_urls("hyprland");

        assert!(
            urls.iter()
                .any(|u| u.contains("declarch-packages/main/modules/hyprland.kdl"))
        );
    }

    #[test]
    fn test_build_urls_default_registry() {
        let urls = build_urls("hyprland/niri-nico");

        // Should try both GitHub and default registry
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com")));
        assert!(
            urls.iter()
                .any(|u| u.contains("declarch-packages/main/modules"))
        );
    }

    #[test]
    fn test_build_urls_colon_suffix() {
        let urls = build_urls("myuser/dotfiles:uwsm");

        // Should build URLs with declarch-uwsm.kdl
        assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| {
            u.contains("raw.githubusercontent.com/myuser/dotfiles/main/declarch-uwsm.kdl")
        }));
    }

    #[test]
    fn test_build_urls_colon_suffix_with_branch() {
        let urls = build_urls("myuser/dotfiles:develop");

        // Should try both develop and main/master branches
        assert!(urls.iter().any(|u| u.contains("declarch-develop.kdl")));
        assert!(
            urls.iter()
                .any(|u| u.contains("/main/declarch-develop.kdl"))
        );
        assert!(
            urls.iter()
                .any(|u| u.contains("/master/declarch-develop.kdl"))
        );
    }

    #[test]
    fn test_build_urls_colon_suffix_main_variant() {
        let urls = build_urls("myuser/dotfiles:main");

        // Should not duplicate - "main" is the default branch
        assert!(urls.iter().any(|u| u.contains("declarch-main.kdl")));
        assert_eq!(urls.len(), 1); // Only one URL since main is default
    }

    #[test]
    fn test_build_urls_registry_with_kdl_extension() {
        let urls = build_urls("gaming/steam-setup.kdl");

        // Registry paths with .kdl should keep the extension
        assert!(
            urls.iter()
                .any(|u| u.contains("declarch-packages/main/modules/gaming/steam-setup.kdl"))
        );
        // Should also try GitHub as fallback
        assert!(
            urls.iter()
                .any(|u| u
                    .contains("raw.githubusercontent.com/gaming/steam-setup/main/declarch.kdl"))
        );
    }

    #[test]
    fn test_build_urls_branch_with_variant() {
        let urls = build_urls("myuser/dotfiles/develop:uwsm");

        // Should build URLs with develop branch and uwsm variant
        assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| {
            u.contains("raw.githubusercontent.com/myuser/dotfiles/develop/declarch-uwsm.kdl")
        }));
        // Should also fallback to main/master
        assert!(urls.iter().any(|u| u.contains("/main/declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| u.contains("/master/declarch-uwsm.kdl")));
    }

    #[test]
    fn test_private_address_checks() {
        assert!(is_private_address("localhost"));
        assert!(is_private_address("127.0.0.1"));
        assert!(is_private_address("192.168.1.1"));
        assert!(is_private_address("10.0.0.5"));
        assert!(is_private_address("172.16.0.1"));
        assert!(is_private_address("172.31.255.255"));
        assert!(is_private_address("169.254.169.254")); // Cloud metadata
        assert!(is_private_address("fe80::1")); // IPv6 Link-local
        assert!(is_private_address("fc00::")); // IPv6 ULA

        assert!(!is_private_address("172.32.0.1"));
        assert!(!is_private_address("8.8.8.8"));
        assert!(!is_private_address("1.1.1.1"));
        assert!(!is_private_address("github.com"));
        assert!(!is_private_address("gitlab.com"));
        assert!(!is_private_address("fcdn.example.com"));
    }

    #[test]
    fn test_validate_url_rejects_malformed_host() {
        assert!(validate_url("https://").is_err());
    }

    #[test]
    fn test_validate_url_rejects_http_by_default() {
        assert!(validate_url("http://example.com/config.kdl").is_err());
    }

    #[test]
    fn test_first_private_ip_detects_private_from_resolved_set() {
        let addrs = vec![
            "8.8.8.8".parse::<IpAddr>().expect("parse public ip"),
            "10.0.0.1".parse::<IpAddr>().expect("parse private ip"),
        ];
        assert_eq!(
            first_private_ip(&addrs).expect("private ip expected"),
            "10.0.0.1".parse::<IpAddr>().expect("parse private ip")
        );
    }

    #[test]
    fn test_failure_summary_formats_attempts() {
        let summary = format_failure_summary(&[
            "- u1 => e1".to_string(),
            "- u2 => e2".to_string(),
            "- u3 => e3".to_string(),
            "- u4 => e4".to_string(),
        ]);

        assert!(summary.contains("Fetch attempts:"));
        assert!(summary.contains("u1"));
        assert!(summary.contains("... and 1 more"));
    }
}
