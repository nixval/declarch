use crate::error::{DeclarchError, Result};
use crate::ui as output;
use reqwest::blocking::Client;

const REPO_BASE_URL: &str = "https://raw.githubusercontent.com/nixval/declarch-packages/main";

/// Fetch module content from remote registry
///
/// Supports contributor-based structure:
/// - "hyprland/niri-nico" → modules/hyprland/niri-nico.kdl
/// - "gaming/steam-setup" → modules/gaming/steam-setup.kdl
///
/// Each category (hyprland, gaming, etc.) can have multiple contributor configs.
pub fn fetch_module_content(module_path: &str) -> Result<String> {
    let client = Client::new();

    // Normalize path: hyprland/niri-nico → modules/hyprland/niri-nico.kdl
    let url_path = if module_path.contains('/') {
        // Nested path: wm/hyprland → modules/wm/hyprland.kdl
        if module_path.ends_with(".kdl") {
            format!("modules/{}", module_path)
        } else {
            format!("modules/{}.kdl", module_path)
        }
    } else {
        // Flat path: hyprland → modules/hyprland.kdl
        if module_path.ends_with(".kdl") {
            format!("modules/{}", module_path)
        } else {
            format!("modules/{}.kdl", module_path)
        }
    };

    let url = format!("{}/{}", REPO_BASE_URL, url_path);

    output::info(&format!("Fetching from: {}", url));

    match fetch_url(&client, &url) {
        Ok(content) => {
            // Add header comment
            let mut final_content = String::new();
            final_content.push_str(&format!("// Module: {}\n", module_path));
            final_content.push_str("// Source: declarch-packages registry\n");
            final_content.push_str(&format!("// URL: {}\n", url));
            final_content.push_str("\n");
            final_content.push_str(&content);

            Ok(final_content)
        },
        Err(_e) => {
            Err(DeclarchError::TargetNotFound(format!(
                "Module '{}' not found in registry\n  Tried: {}",
                module_path, url
            )))
        }
    }
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
