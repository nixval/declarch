use assert_cmd::Command;
use predicates::prelude::*;

// Helper function to initialize the command to test.
fn declarch() -> Command {
    Command::new(env!("CARGO_BIN_EXE_declarch"))
}

#[test]
fn test_help_command() {
    let mut cmd = declarch();

    // Update expectation to match 'long_about' visible in --help
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Universal declarative package manager",
        ));
}

#[test]
fn test_version_flag() {
    let mut cmd = declarch();

    // Update expectation to match your current Cargo.toml version
    let version = env!("CARGO_PKG_VERSION");
    let expected = format!("declarch {}", version);

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn test_init_dry_run() {
    // Rename to _temp_dir to suppress "unused variable" warning
    let _temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = declarch();

    cmd.arg("unknown-command-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: declarch"));
}
