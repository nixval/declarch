use super::diagnostics;
use crate::core::types::Backend;
use crate::state::types::{PackageState, State};
use chrono::Utc;

fn sample_state() -> State {
    State::default()
}

fn pkg(backend: &str, config_name: &str) -> PackageState {
    PackageState {
        backend: Backend::from(backend),
        config_name: config_name.to_string(),
        provides_name: config_name.to_string(),
        actual_package_name: None,
        installed_at: Utc::now(),
        version: None,
        install_reason: None,
        source_module: None,
        last_seen_at: None,
        backend_meta: None,
    }
}

#[test]
fn state_signature_duplicates_detected() {
    let mut state = sample_state();
    state
        .packages
        .insert("legacy:bat".to_string(), pkg("aur", "bat"));
    state
        .packages
        .insert("aur:bat".to_string(), pkg("aur", "bat"));
    state
        .packages
        .insert("aur:ripgrep".to_string(), pkg("aur", "ripgrep"));

    let duplicates = diagnostics::collect_state_signature_duplicates(&state);
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].0, "aur:bat");
    assert_eq!(duplicates[0].1.len(), 2);
}

#[test]
fn state_signature_duplicates_empty_when_clean() {
    let mut state = sample_state();
    state
        .packages
        .insert("aur:bat".to_string(), pkg("aur", "bat"));
    state
        .packages
        .insert("aur:ripgrep".to_string(), pkg("aur", "ripgrep"));

    let duplicates = diagnostics::collect_state_signature_duplicates(&state);
    assert!(duplicates.is_empty());
}
