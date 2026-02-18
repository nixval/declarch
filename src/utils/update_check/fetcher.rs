use crate::project_identity;
use crate::utils::update_check::versioning::version_tuple;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub(super) fn fetch_latest_version(timeout: Duration) -> Option<String> {
    let client = Client::builder().timeout(timeout).build().ok()?;

    let response = client
        .get(project_identity::github_latest_release_api())
        .header(
            "User-Agent",
            format!("{}-cli", project_identity::BINARY_NAME),
        )
        .send()
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body = response.text().ok()?;
    parse_latest_version_from_body(&body)
}

pub(super) fn parse_latest_version_from_body(body: &str) -> Option<String> {
    let release: GitHubRelease = serde_json::from_str(body).ok()?;
    let tag = release.tag_name.trim();
    let normalized = tag.strip_prefix('v').unwrap_or(tag).to_string();
    if version_tuple(&normalized).is_some() {
        Some(normalized)
    } else {
        None
    }
}
