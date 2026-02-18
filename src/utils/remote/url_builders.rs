use super::{BACKENDS_REGISTRY, DEFAULT_REGISTRY};
use crate::constants::{CONFIG_EXTENSION, DEFAULT_BRANCHES, PROJECT_NAME};

pub(super) fn build_backend_urls(backend_name: &str) -> Vec<String> {
    let mut urls = Vec::new();

    if backend_name.starts_with("http://") || backend_name.starts_with("https://") {
        urls.push(backend_name.to_string());
        return urls;
    }

    let clean_name = backend_name
        .strip_suffix(&format!(".{}", CONFIG_EXTENSION))
        .unwrap_or(backend_name);

    urls.push(format!(
        "{}/{}.{}",
        *BACKENDS_REGISTRY, clean_name, CONFIG_EXTENSION
    ));

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

pub(super) fn build_urls(target: &str) -> Vec<String> {
    let mut urls = Vec::new();

    if target.starts_with("http://") || target.starts_with("https://") {
        urls.push(target.to_string());
        return urls;
    }

    let clean_target = target
        .strip_suffix(&format!(".{}", CONFIG_EXTENSION))
        .unwrap_or(target);

    if clean_target.contains(':') && !clean_target.starts_with("gitlab.com/") {
        let parts: Vec<&str> = clean_target.split(':').collect();

        if parts.len() == 2 {
            let (repo_part, variant) = (parts[0], parts[1]);

            if repo_part.contains('/') {
                let repo_parts: Vec<&str> = repo_part.split('/').collect();

                if repo_parts.len() == 2 {
                    let (owner, repo_name) = (repo_parts[0], repo_parts[1]);

                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/main/{}-{}.{}",
                        owner, repo_name, PROJECT_NAME, variant, CONFIG_EXTENSION
                    ));

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
                    let (owner, repo_name, branch) = (repo_parts[0], repo_parts[1], repo_parts[2]);

                    urls.push(format!(
                        "https://raw.githubusercontent.com/{}/{}/{}/{}-{}.{}",
                        owner, repo_name, branch, PROJECT_NAME, variant, CONFIG_EXTENSION
                    ));

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

    if clean_target.starts_with("gitlab.com/") {
        let path = clean_target
            .strip_prefix("gitlab.com/")
            .unwrap_or(clean_target);
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() >= 2 {
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

    if target.contains('/') && target.ends_with(&format!(".{}", CONFIG_EXTENSION)) {
        urls.push(format!("{}/modules/{}", *DEFAULT_REGISTRY, target));

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

    if clean_target.contains('/') {
        let parts: Vec<&str> = clean_target.split('/').collect();

        if parts.len() >= 2 {
            let (owner, repo, branch) = if parts.len() >= 3 {
                (parts[0], parts[1], parts[2])
            } else {
                (parts[0], parts[1], "main")
            };

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

            urls.push(format!(
                "{}/modules/{}.{}",
                *DEFAULT_REGISTRY, clean_target, CONFIG_EXTENSION
            ));
        }

        return urls;
    }

    urls.push(format!(
        "{}/modules/{}.{}",
        *DEFAULT_REGISTRY, clean_target, CONFIG_EXTENSION
    ));

    urls
}
