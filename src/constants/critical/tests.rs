use super::*;

#[test]
fn test_all_critical_count() {
    assert_eq!(ALL.len(), 45);
}

#[test]
fn test_contains_linux() {
    assert!(ALL.contains(&"linux"));
}

#[test]
fn test_contains_systemd() {
    assert!(ALL.contains(&"systemd"));
}

#[test]
fn test_contains_declarch() {
    assert!(ALL.contains(&"declarch"));
    assert!(ALL.contains(&"declarch-bin"));
}

#[test]
fn test_not_contains_paru() {
    assert!(!ALL.contains(&"paru"));
}

#[test]
fn test_not_contains_yay() {
    assert!(!ALL.contains(&"yay"));
}

#[test]
fn test_not_contains_git() {
    assert!(!ALL.contains(&"git"));
}
