use crate::error::{DeclarchError, Result};
use reqwest::blocking::Client;

const REPO_BASE_URL: &str = "https://raw.githubusercontent.com/nixval/declarch-packages/main";

pub fn fetch_module_content(module_name: &str) -> Result<String> {
    let client = Client::new();
    let mut final_content = String::new();
    
    // Header Info
    final_content.push_str(&format!("// Module: {}\n", module_name));
    final_content.push_str("// Source: declarch-packages (Remote)\n\n");

    // 1. Fetch 'default.kdl'
    let default_url = format!("{}/modules/{}/default.kdl", REPO_BASE_URL, module_name);
    if let Ok(content) = fetch_url(&client, &default_url) {
        final_content.push_str("// --- Common Packages ---\n");
        final_content.push_str(&content);
        final_content.push_str("\n\n");
    }

    // 2. Fetch 'arch.kdl'
    let arch_url = format!("{}/modules/{}/arch.kdl", REPO_BASE_URL, module_name);
    match fetch_url(&client, &arch_url) {
        Ok(content) => {
            final_content.push_str("// --- Arch Specific ---\n");
            final_content.push_str(&content);
            final_content.push_str("\n");
        },
        Err(_) => {
            if final_content.len() < 100 { 
                return Err(DeclarchError::TargetNotFound(format!(
                    "Module '{}' not found in registry (checked arch.kdl and default.kdl)", 
                    module_name
                )));
            }
        }
    }

    Ok(final_content)
}

fn fetch_url(client: &Client, url: &str) -> Result<String> {
    let resp = client.get(url)
        .header("User-Agent", "declarch-cli")
        .send()
        .map_err(|e| DeclarchError::Other(format!("Network error: {}", e)))?;

    if resp.status().is_success() {
        Ok(resp.text().unwrap_or_default())
    } else {
        Err(DeclarchError::Other("Remote file not found".into()))
    }
}
