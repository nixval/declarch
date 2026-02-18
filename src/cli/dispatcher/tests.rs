use super::normalization::parse_limit_option;
use super::output_contract::validate_machine_output_contract;
use crate::cli::args::{Cli, GlobalFlags};

fn base_cli() -> Cli {
    Cli {
        global: GlobalFlags {
            verbose: false,
            quiet: false,
            yes: false,
            force: false,
            dry_run: false,
            format: None,
            output_version: None,
        },
        command: None,
    }
}

#[test]
fn output_version_v1_allows_json_format() {
    use crate::cli::args::{Command, LintMode};
    let mut cli = base_cli();
    cli.global.output_version = Some("v1".to_string());
    cli.global.format = Some("json".to_string());
    cli.command = Some(Command::Lint {
        strict: false,
        fix: false,
        mode: LintMode::All,
        backend: None,
        diff: false,
        benchmark: false,
        repair_state: false,
        state_rm: Vec::new(),
        state_rm_backend: None,
        state_rm_all: false,
        profile: None,
        host: None,
        modules: Vec::new(),
    });
    assert!(validate_machine_output_contract(&cli).is_ok());
}

#[test]
fn output_version_rejects_unknown_version() {
    let mut cli = base_cli();
    cli.global.output_version = Some("v2".to_string());
    cli.global.format = Some("json".to_string());
    assert!(validate_machine_output_contract(&cli).is_err());
}

#[test]
fn output_version_requires_structured_format() {
    let mut cli = base_cli();
    cli.global.output_version = Some("v1".to_string());
    cli.global.format = Some("table".to_string());
    assert!(validate_machine_output_contract(&cli).is_err());
}

#[test]
fn output_version_rejects_unsupported_command() {
    use crate::cli::args::{Command, SyncCommand};
    let mut cli = base_cli();
    cli.global.output_version = Some("v1".to_string());
    cli.global.format = Some("json".to_string());
    cli.command = Some(Command::Sync {
        target: None,
        diff: false,
        noconfirm: false,
        hooks: false,
        profile: None,
        host: None,
        modules: Vec::new(),
        command: Some(SyncCommand::Update {
            target: None,
            diff: false,
            noconfirm: false,
            hooks: false,
            profile: None,
            host: None,
            modules: Vec::new(),
        }),
    });
    assert!(validate_machine_output_contract(&cli).is_err());
}

#[test]
fn output_version_allows_dry_run_sync() {
    use crate::cli::args::Command;
    let mut cli = base_cli();
    cli.global.output_version = Some("v1".to_string());
    cli.global.format = Some("json".to_string());
    cli.global.dry_run = true;
    cli.command = Some(Command::Sync {
        target: None,
        diff: false,
        noconfirm: false,
        hooks: false,
        profile: None,
        host: None,
        modules: Vec::new(),
        command: None,
    });
    assert!(validate_machine_output_contract(&cli).is_ok());
}

#[test]
fn parse_limit_option_defaults_to_ten() {
    assert_eq!(parse_limit_option(None).unwrap(), Some(10));
}

#[test]
fn parse_limit_option_supports_unlimited() {
    assert_eq!(parse_limit_option(Some("0")).unwrap(), None);
    assert_eq!(parse_limit_option(Some("all")).unwrap(), None);
}

#[test]
fn parse_limit_option_rejects_invalid_input() {
    assert!(parse_limit_option(Some("abc")).is_err());
}
