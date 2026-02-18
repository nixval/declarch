use super::Cli;
use crate::project_identity;
use clap::{CommandFactory, Parser};

#[test]
fn parser_rejects_removed_sync_preview_subcommand() {
    let parsed = Cli::try_parse_from([project_identity::BINARY_NAME, "sync", "preview"]);
    assert!(parsed.is_err());
}

#[test]
fn parser_rejects_removed_sync_sync_subcommand() {
    let parsed = Cli::try_parse_from([project_identity::BINARY_NAME, "sync", "sync"]);
    assert!(parsed.is_err());
}

#[test]
fn parser_allows_switch_with_global_dry_run() {
    let parsed = Cli::try_parse_from([
        project_identity::BINARY_NAME,
        "switch",
        "old",
        "new",
        "--dry-run",
    ])
    .expect("switch with global --dry-run should parse");
    assert!(parsed.global.dry_run);
}

#[test]
fn help_sync_no_longer_shows_removed_preview_or_gc() {
    let mut cmd = Cli::command();
    let sync = cmd
        .find_subcommand_mut("sync")
        .expect("sync subcommand exists");
    let mut out = Vec::new();
    sync.write_long_help(&mut out)
        .expect("can render sync help");
    let help = String::from_utf8(out).expect("help is valid utf8");
    assert!(!help.contains("sync preview"));
    assert!(!help.contains("--gc"));
    assert!(help.contains("update"));
    assert!(help.contains("prune"));
}

#[test]
fn help_info_uses_scope_not_legacy_list_flags() {
    let mut cmd = Cli::command();
    let info = cmd
        .find_subcommand_mut("info")
        .expect("info subcommand exists");
    let mut out = Vec::new();
    info.write_long_help(&mut out)
        .expect("can render info help");
    let help = String::from_utf8(out).expect("help is valid utf8");
    assert!(help.contains("--scope"));
    assert!(help.contains("unmanaged"));
    assert!(!help.contains("--orphans"));
    assert!(!help.contains("--synced"));
}

#[test]
fn help_does_not_show_self_update() {
    let mut cmd = Cli::command();
    let mut out = Vec::new();
    cmd.write_long_help(&mut out).expect("can render root help");
    let help = String::from_utf8(out).expect("help is valid utf8");
    assert!(!help.contains("self-update"));
    assert!(!help.contains("selfupdate"));
}
