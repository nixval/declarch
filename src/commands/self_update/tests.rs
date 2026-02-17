use super::*;

#[cfg(not(target_os = "windows"))]
#[test]
fn parse_checksum_line_supports_coreutils_and_bsd_style() {
    let file_name = format!(
        "{}-x86_64-unknown-linux-gnu.tar.gz",
        project_identity::RELEASE_ASSET_PREFIX
    );
    assert_eq!(
        parse_checksum_line(&format!("abc123  {}", file_name)),
        Some(("abc123", file_name.as_str()))
    );
    assert_eq!(
        parse_checksum_line(&format!("abc123 *{}", file_name)),
        Some(("abc123", file_name.as_str()))
    );
}

#[cfg(not(target_os = "windows"))]
#[test]
fn verify_checksum_detects_mismatch() {
    let file_name = format!(
        "{}-x86_64-unknown-linux-gnu.tar.gz",
        project_identity::RELEASE_ASSET_PREFIX
    );
    let result = verify_checksum(&file_name, "abc123", "def456");
    assert!(result.is_err());
}

#[test]
fn build_windows_update_bootstrap_contains_expected_values() {
    let ps = build_windows_update_bootstrap_ps("0.8.2");
    assert!(ps.contains("Start-Process -WindowStyle Hidden"));
    assert!(ps.contains("install.ps1"));
    assert!(ps.contains("/v0.8.2/install.ps1"));
    assert!(ps.contains("-Version '0.8.2'"));
    assert!(ps.contains(project_identity::REPO_SLUG));
}
