use super::*;

#[test]
fn build_program_non_elevated_works() {
    let cmd = build_program_command("echo", &["ok".to_string()], false).unwrap();
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("echo"));
}

#[test]
fn build_shell_non_elevated_works() {
    let cmd = build_shell_command("echo ok", false).unwrap();
    let debug = format!("{:?}", cmd);
    #[cfg(unix)]
    assert!(debug.contains("\"sh\""));
}

#[test]
fn current_os_tag_is_not_empty() {
    assert!(!current_os_tag().is_empty());
}
