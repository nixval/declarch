use crate::error::{DeclarchError, Result};
use crate::ui as output;
use reqwest::blocking::Client;
use std::time::Duration;

const DEFAULT_REGISTRY: &str = "https://raw.githubusercontent.com/nixval/declarch-packages/main";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Allowed URL schemes for security (prevent SSRF)
const ALLOWED_SCHEMES: &[&str] = &["http", "https"];

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
        .map_err(|e| DeclarchError::Other(format!("Failed to create HTTP client: {}", e)))?;

    // Try different URL patterns
    let urls = build_urls(target_path);

    for url in urls {
        output::info(&format!("Trying: {}", url));

        match fetch_url(&client, &url) {
            Ok(content) => {
                // Add header comment
                let mut final_content = String::new();
                final_content.push_str(&format!("// Source: {}\n", target_path));
                final_content.push_str(&format!("// URL: {}\n", url));
                final_content.push_str("\n");
                final_content.push_str(&content);

                return Ok(final_content);
            },
            Err(_) => continue,
        }
    }

    Err(DeclarchError::TargetNotFound(format!(
        "Failed to fetch from: {}\n  Hint: Ensure the repository has a declarch.kdl file",
        target_path
    )))
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
    let clean_target = target.strip_suffix(".kdl").unwrap_or(target);

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
                        "https://raw.githubusercontent.com/{}/{}/main/declarch-{}.kdl",
                        owner, repo_name, variant
                    ));

                    // Also try branches (main, master)
                    if variant != "main" && variant != "master" {
                        for b in ["main", "master"] {
                            urls.push(format!(
                                "https://raw.githubusercontent.com/{}/{}/{}/declarch-{}.kdl",
                                owner, repo_name, b, variant
                            ));
                        }
                    }

                    return urls;
                } else if repo_parts.len() >= 3 {
                    // user/repo/branch:variant pattern
                    let (owner, repo_name, branch) = (repo_parts[0], repo_parts[1], repo_parts[2]);

                    // Try: user/repo/branch:variant → user/repo/branch/declarch-variant.kdl
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/declarch-{}.kdl",
                        owner, repo_name, branch, variant
                    ));

                    // Also try main/master as fallback
                    if branch != "main" && branch != "master" {
                        for b in ["main", "master"] {
                            urls.push(format!(
                                "https://raw.githubusercontent.com/{}/{}/{}/declarch-{}.kdl",
                                owner, repo_name, b, variant
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
                "https://gitlab.com/{}/{}/-/raw/{}/declarch.kdl",
                owner, repo, branch
            ));
        }

        return urls;
    }

    // 4. Registry module path (has .kdl extension, like "gaming/steam-setup.kdl")
    // This takes priority over GitHub user/repo to avoid ambiguity
    if target.contains('/') && target.ends_with(".kdl") {
        // It's a registry module path with extension
        urls.push(format!(
            "{}/modules/{}",
            DEFAULT_REGISTRY, target  // Keep .kdl extension
        ));

        // Also try GitHub as fallback
        let parts: Vec<&str> = clean_target.split('/').collect();
        if parts.len() >= 2 {
            let (owner, repo) = (parts[0], parts[1]);
            urls.push(format!(
                "https://raw.githubusercontent.com/{}/{}/main/declarch.kdl",
                owner, repo
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
                vec![branch, "main", "master"]
            };

            for b in branches_to_try {
                urls.push(format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/declarch.kdl",
                    owner, repo, b
                ));
            }

            // Also try modules/ path for default registry (without .kdl)
            urls.push(format!(
                "{}/modules/{}.kdl",
                DEFAULT_REGISTRY, clean_target
            ));
        }

        return urls;
    }

    // 6. Flat name → try default registry
    urls.push(format!("{}/modules/{}.kdl", DEFAULT_REGISTRY, clean_target));

    urls
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
    // Validate URL scheme before making request
    validate_url(url)?;

    let resp = client.get(url)
        .header("User-Agent", "declarch-cli")
        .send()
        .map_err(|e| DeclarchError::Other(format!("Network error: {}", e)))?;

    if resp.status().is_success() {
        Ok(resp.text().unwrap_or_default())
    } else {
        Err(DeclarchError::Other(format!("HTTP {}", resp.status())))
    }
}

/// Validate URL to prevent SSRF attacks
fn validate_url(url_str: &str) -> Result<()> {
    // Parse URL to validate scheme
    let parsed = reqwest::Url::parse(url_str)
        .map_err(|_| DeclarchError::Other(format!("Invalid URL: {}", url_str)))?;

    // Check scheme is allowed
    let scheme = parsed.scheme();
    if !ALLOWED_SCHEMES.contains(&scheme) {
        return Err(DeclarchError::Other(format!(
            "URL scheme '{}' not allowed. Only HTTP and HTTPS are permitted.",
            scheme
        )));
    }

    // Prevent access to localhost/private networks (basic check)
    let host = parsed.host_str().unwrap_or("");
    if is_private_address(host) {
        return Err(DeclarchError::Other(format!(
            "Access to private addresses is not allowed: {}",
            host
        )));
    }

    Ok(())
}

/// Check if hostname is a private/local address
fn is_private_address(host: &str) -> bool {
    // Check for localhost variants
    if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "0.0.0.0" {
        return true;
    }

    // Check for private IP ranges (basic check for common patterns)
    if host.starts_with("127.") || host.starts_with("10.") || host.starts_with("192.168.") {
        return true;
    }

    // Check for 172.16.0.0/12 (172.16.x.x to 172.31.x.x)
    if host.starts_with("172.") {
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() >= 2 {
            if let Ok(second_octet) = parts[1].parse::<u8>() {
                if (16..=32).contains(&second_octet) {
                    return true;
                }
            }
        }
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

        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/develop/declarch.kdl")));
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/hyprland1/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_gitlab() {
        let urls = build_urls("gitlab.com/user/repo");
        
        assert!(urls.iter().any(|u| u.contains("gitlab.com/user/repo/-/raw/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_gitlab_with_branch() {
        let urls = build_urls("gitlab.com/user/repo/develop");
        
        assert!(urls.iter().any(|u| u.contains("gitlab.com/user/repo/-/raw/develop/declarch.kdl")));
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

        assert!(urls.iter().any(|u| u.contains("declarch-packages/main/modules/hyprland.kdl")));
    }

    #[test]
    fn test_build_urls_default_registry() {
        let urls = build_urls("hyprland/niri-nico");

        // Should try both GitHub and default registry
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com")));
        assert!(urls.iter().any(|u| u.contains("declarch-packages/main/modules")));
    }

    #[test]
    fn test_build_urls_colon_suffix() {
        let urls = build_urls("myuser/dotfiles:uwsm");

        // Should build URLs with declarch-uwsm.kdl
        assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/dotfiles/main/declarch-uwsm.kdl")));
    }

    #[test]
    fn test_build_urls_colon_suffix_with_branch() {
        let urls = build_urls("myuser/dotfiles:develop");

        // Should try both develop and main/master branches
        assert!(urls.iter().any(|u| u.contains("declarch-develop.kdl")));
        assert!(urls.iter().any(|u| u.contains("/main/declarch-develop.kdl")));
        assert!(urls.iter().any(|u| u.contains("/master/declarch-develop.kdl")));
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
        assert!(urls.iter().any(|u| u.contains("declarch-packages/main/modules/gaming/steam-setup.kdl")));
        // Should also try GitHub as fallback
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/gaming/steam-setup/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_branch_with_variant() {
        let urls = build_urls("myuser/dotfiles/develop:uwsm");

        // Should build URLs with develop branch and uwsm variant
        assert!(urls.iter().any(|u| u.contains("declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/myuser/dotfiles/develop/declarch-uwsm.kdl")));
        // Should also fallback to main/master
        assert!(urls.iter().any(|u| u.contains("/main/declarch-uwsm.kdl")));
        assert!(urls.iter().any(|u| u.contains("/master/declarch-uwsm.kdl")));
    }
}
