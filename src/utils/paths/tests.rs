use super::*;
use crate::constants::{CONFIG_FILE_NAME, STATE_FILE_NAME};

#[test]
fn config_file_uses_expected_filename() {
    let path = config_file().expect("config_file should resolve");
    assert_eq!(
        path.file_name().and_then(|f| f.to_str()),
        Some(CONFIG_FILE_NAME)
    );
}

#[test]
fn state_file_uses_expected_filename() {
    let path = state_file().expect("state_file should resolve");
    assert_eq!(
        path.file_name().and_then(|f| f.to_str()),
        Some(STATE_FILE_NAME)
    );
}
