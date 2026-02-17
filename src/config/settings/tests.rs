use super::*;

#[test]
fn test_defaults() {
    let defaults = Settings::defaults();
    assert_eq!(defaults.get("color"), Some(&"auto".to_string()));
    assert_eq!(defaults.get("progress"), Some(&"off".to_string()));
    assert_eq!(defaults.get("format"), Some(&"table".to_string()));
    assert_eq!(defaults.get("verbose"), Some(&"false".to_string()));
    assert_eq!(defaults.get("compact"), Some(&"false".to_string()));
}

#[test]
fn test_validate_key_valid() {
    let settings = Settings {
        settings_file: PathBuf::from("/tmp/test.kdl"),
        values: HashMap::new(),
    };

    assert!(settings.validate_key("color").is_ok());
    assert!(settings.validate_key("progress").is_ok());
    assert!(settings.validate_key("format").is_ok());
    assert!(settings.validate_key("verbose").is_ok());
}

#[test]
fn test_validate_key_invalid() {
    let settings = Settings {
        settings_file: PathBuf::from("/tmp/test.kdl"),
        values: HashMap::new(),
    };

    assert!(settings.validate_key("invalid").is_err());
}

#[test]
fn test_validate_value_color() {
    let settings = Settings {
        settings_file: PathBuf::from("/tmp/test.kdl"),
        values: HashMap::new(),
    };

    assert!(settings.validate_value("color", "auto").is_ok());
    assert!(settings.validate_value("color", "always").is_ok());
    assert!(settings.validate_value("color", "never").is_ok());
    assert!(settings.validate_value("color", "invalid").is_err());
}
