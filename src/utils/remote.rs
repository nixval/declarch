use crate::error::{DeclarchError, Result};
use crate::ui as output;
use reqwest::blocking::Client;

const DEFAULT_REGISTRY: &str = "https://raw.githubusercontent.com/nixval/declarch-packages/main";

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
///    declarch init jakoolit/hyprland1
///    # → https://raw.githubusercontent.com/jakoolit/hyprland1/main/declarch.kdl
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
    let client = Client::new();

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

    // 1. Direct URL (starts with http:// or https://)
    if target.starts_with("http://") || target.starts_with("https://") {
        urls.push(target.to_string());
        return urls;
    }

    // 2. GitLab URL (gitlab.com/user/repo or gitlab.com/user/repo/branch)
    if target.starts_with("gitlab.com/") {
        let path = target.strip_prefix("gitlab.com/").unwrap();
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

    // 3. GitHub-style URLs (owner/repo or owner/repo/branch)
    if target.contains('/') {
        let parts: Vec<&str> = target.split('/').collect();

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

            // Also try modules/ path for default registry
            urls.push(format!(
                "{}/modules/{}.kdl",
                DEFAULT_REGISTRY, target
            ));
        }

        return urls;
    }

    // 4. Flat name → try default registry
    urls.push(format!("{}/modules/{}.kdl", DEFAULT_REGISTRY, target));

    urls
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_urls_github_default() {
        let urls = build_urls("jakoolit/hyprland1");

        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/jakoolit/hyprland1/main/declarch.kdl")));
    }

    #[test]
    fn test_build_urls_github_with_branch() {
        let urls = build_urls("jakoolit/hyprland1/develop");

        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/jakoolit/hyprland1/develop/declarch.kdl")));
        assert!(urls.iter().any(|u| u.contains("raw.githubusercontent.com/jakoolit/hyprland1/main/declarch.kdl")));
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
}
