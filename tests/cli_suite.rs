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

#[test]
fn output_contract_rejects_unsupported_sync_machine_mode() {
    let mut cmd = declarch();
    cmd.args(["sync", "--format", "json", "--output-version", "v1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "does not support --output-version v1 yet",
        ));
}

#[test]
fn output_contract_requires_structured_format() {
    let mut cmd = declarch();
    cmd.args([
        "search",
        "firefox",
        "--format",
        "table",
        "--output-version",
        "v1",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "--output-version v1 requires --format json|yaml",
    ));
}

#[test]
fn output_contract_rejects_unsupported_version() {
    let mut cmd = declarch();
    cmd.args([
        "search",
        "firefox",
        "--format",
        "json",
        "--output-version",
        "v2",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Unsupported output contract version",
    ));
}
