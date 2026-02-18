use super::*;

#[test]
fn test_default_config() {
    let config = GlobalConfig::default();
    assert_eq!(config.aur_helper, AurHelper::Paru);
}

#[test]
fn test_aur_helper_display() {
    assert_eq!(AurHelper::Paru.to_string(), "paru");
    assert_eq!(AurHelper::Yay.to_string(), "yay");
}
