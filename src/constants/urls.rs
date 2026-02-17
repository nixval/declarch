/// External URLs and URL patterns
///
/// Centralized URLs for:
/// - Default registry (declarch-packages)
/// - Remote init (GitHub, GitLab patterns)
/// - Repository documentation
/// - Installation guides
///
/// Benefits:
/// - Easy to change default registry
/// - Reusable for docs generation
/// - Consistent URL patterns
///
/// Default remote registry for declarch configurations
///
/// Used by `declarch init <target>` when no URL is provided.
/// Format: https://raw.githubusercontent.com/nixval/declarch-packages/main
pub const DEFAULT_REGISTRY: &str =
    "https://raw.githubusercontent.com/nixval/declarch-packages/main";

/// Repository documentation URLs
pub mod repos {
    pub const AUR: &str = "https://aur.archlinux.org";
    pub const AUR_SEARCH: &str = "https://aur.archlinux.org/packages";
    pub const AUR_WIKI: &str = "https://wiki.archlinux.org";

    pub const FLATHUB: &str = "https://flathub.org";
    pub const FLATHUB_REPO: &str = "https://flathub.org/repo/flathub.flatpakrepo";

    pub const NPM: &str = "https://www.npmjs.com";
    pub const NPM_SEARCH: &str = "https://www.npmjs.com/search";

    pub const PYPI: &str = "https://pypi.org";
    pub const PYPI_SEARCH: &str = "https://pypi.org/search";

    pub const CRATES_IO: &str = "https://crates.io";
    pub const CRATES_IO_SEARCH: &str = "https://crates.io/search";

    pub const SOAR: &str = "https://soar.dev";
    pub const SOAR_SEARCH: &str = "https://soar.dev/search";
}

/// Installation guides and help URLs
pub mod installation {
    pub const SOAR_INSTALL: &str = "https://github.com/pkgforge/soar#installation";
}

/// URL builder for remote init
///
/// Handles multiple URL patterns:
/// 1. Default registry (declarch-packages)
/// 2. GitHub repos (user/repo)
/// 3. GitHub repos with branch (user/repo/branch)
/// 4. GitHub repos with variant (user/repo:variant)
/// 5. GitLab repos
///
/// Examples:
/// - `hyprland/niri-nico` -> https://raw.githubusercontent.com/nixval/declarch-packages/main/modules/hyprland/niri-nico.kdl
/// - `myuser/dotfiles` -> https://raw.githubusercontent.com/myuser/dotfiles/main/declarch.kdl
/// - `gitlab.com/user/repo` -> https://gitlab.com/user/repo/-/raw/main/declarch.kdl
pub struct RemoteUrlBuilder {
    pub registry: String,
    pub timeout_secs: u64,
}

impl Default for RemoteUrlBuilder {
    fn default() -> Self {
        Self {
            registry: project_identity::registry_raw_base_url(),
            timeout_secs: 30,
        }
    }
}

impl RemoteUrlBuilder {
    /// Build URLs to try for a given target path
    ///
    /// # Examples
    /// ```
    /// # use declarch::constants::urls::RemoteUrlBuilder;
    /// let builder = RemoteUrlBuilder::default();
    /// let urls = builder.build_urls("myuser/dotfiles");
    /// assert!(!urls.is_empty()); // Should generate at least one URL
    /// ```
    pub fn build_urls(&self, target_path: &str) -> Vec<String> {
        let mut urls = Vec::new();

        // 1. Direct URL (http:// or https://)
        if target_path.starts_with("http://") || target_path.starts_with("https://") {
            urls.push(target_path.to_string());
            return urls;
        }

        // Strip .kdl extension for cleaner URL building
        let clean_target = target_path.strip_suffix(".kdl").unwrap_or(target_path);

        // 2. Config variant syntax: user/repo:variant or user/repo/branch:variant
        if clean_target.contains(':') && !clean_target.starts_with("gitlab.com/") {
            urls.extend(self.build_variant_urls(clean_target));
            return urls;
        }

        // 3. GitHub/GitLab repo (user/repo or gitlab.com/user/repo)
        if clean_target.contains('/') {
            urls.extend(self.build_repo_urls(clean_target));
            return urls;
        }

        // 4. Default registry path
        urls.push(format!(
            "{}/{}",
            self.registry.trim_end_matches('/'),
            clean_target
        ));

        urls
    }

    /// Build URLs for variant syntax (user/repo:variant or user/repo/branch:variant)
    fn build_variant_urls(&self, target: &str) -> Vec<String> {
        let mut urls = Vec::new();
        let parts: Vec<&str> = target.split(':').collect();

        if parts.len() == 2 {
            let (repo_part, variant) = (parts[0], parts[1]);

            if repo_part.contains('/') {
                let repo_parts: Vec<&str> = repo_part.split('/').collect();

                if repo_parts.len() == 2 {
                    let (user, repo) = (repo_parts[0], repo_parts[1]);

                    // Default registry with variant
                    urls.push(format!(
                        "{}/modules/{}/{}/{}-{}.kdl",
                        self.registry.trim_end_matches('/'),
                        repo,
                        variant,
                        project_identity::RELEASE_ASSET_PREFIX,
                        variant
                    ));

                    // GitHub pattern with variant (default branch)
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/main/{}-{}.kdl",
                        user,
                        repo,
                        project_identity::RELEASE_ASSET_PREFIX,
                        variant
                    ));

                    // GitHub pattern with variant (variant branch)
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}-{}.kdl",
                        user,
                        repo,
                        variant,
                        project_identity::RELEASE_ASSET_PREFIX,
                        variant
                    ));
                }
            }
        }

        urls
    }

    /// Build URLs for repo syntax (user/repo or gitlab.com/user/repo)
    fn build_repo_urls(&self, target: &str) -> Vec<String> {
        let mut urls = Vec::new();

        if target.starts_with("gitlab.com/") {
            let rest = target.strip_prefix("gitlab.com/").unwrap_or("");
            let parts: Vec<&str> = rest.split('/').collect();

            if parts.len() >= 2 {
                let (user, repo) = (parts[0], parts[1]);

                urls.push(format!(
                    "https://gitlab.com/{}/{}/-/raw/{}/{}",
                    user,
                    repo,
                    "main",
                    project_identity::CONFIG_FILE_BASENAME
                ));
            }
        } else {
            let parts: Vec<&str> = target.split('/').collect();

            if parts.len() >= 2 {
                let (user, repo) = (parts[0], parts[1]);

                // Default registry
                urls.push(format!(
                    "{}/{}/main/{}",
                    self.registry.trim_end_matches('/'),
                    target,
                    project_identity::CONFIG_FILE_BASENAME
                ));

                // GitHub pattern (main branch)
                urls.push(format!(
                    "https://raw.githubusercontent.com/{}/{}/main/{}",
                    user,
                    repo,
                    project_identity::CONFIG_FILE_BASENAME
                ));

                // GitHub pattern (branch)
                if parts.len() >= 3 {
                    let branch = parts[2];
                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}",
                        user,
                        repo,
                        branch,
                        project_identity::CONFIG_FILE_BASENAME
                    ));
                }
            }
        }

        urls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_identity;

    #[test]
    fn test_build_urls_github() {
        let builder = RemoteUrlBuilder::default();
        let urls = builder.build_urls("myuser/dotfiles");

        // Check that we get some URL with github.com (from default registry)
        // OR declarch-packages registry
        assert!(!urls.is_empty());
        assert!(
            urls.iter()
                .any(|u| u.contains(project_identity::CONFIG_FILE_BASENAME))
        );
    }

    #[test]
    fn test_build_urls_variant() {
        let builder = RemoteUrlBuilder::default();
        let urls = builder.build_urls("hyprland/niri-nico");

        // Check that we get a URL containing variant
        assert!(urls.iter().any(|u| u.contains("niri-nico")));
    }

    #[test]
    fn test_build_urls_direct() {
        let builder = RemoteUrlBuilder::default();
        let urls = builder.build_urls("https://example.com/config.kdl");

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/config.kdl");
    }
}
use crate::project_identity;
