use super::normalization::{list_to_optional_vec, map_lint_mode, parse_limit_option};
use crate::cli::args::{Cli, InfoListScope, LintMode, SyncCommand};
use crate::commands;
use crate::error::{DeclarchError, Result};
use crate::project_identity;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_init_command(
    args: &Cli,
    host: &Option<String>,
    path: &Option<String>,
    backend: &[String],
    list: &Option<String>,
    local: bool,
    restore_declarch: bool,
) -> Result<()> {
    if let Some(what) = list {
        return match what.as_str() {
            "backends" => commands::init::list_available_backends(),
            "modules" => commands::init::list_available_modules(),
            _ => Err(DeclarchError::Other(format!(
                "Unknown init list target '{}'. Use '{}' or '{}'.",
                what,
                project_identity::cli_with("init --list backends"),
                project_identity::cli_with("init --list modules"),
            ))),
        };
    }

    if restore_declarch {
        return commands::init::restore_declarch(host.clone());
    }

    commands::init::run(commands::init::InitOptions {
        host: host.clone(),
        path: path.clone(),
        backends: backend.to_vec(),
        force: args.global.force,
        yes: args.global.yes,
        local,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_sync_command(
    args: &Cli,
    target: &Option<String>,
    diff: bool,
    noconfirm: bool,
    hooks: bool,
    profile: &Option<String>,
    host: &Option<String>,
    modules: &[String],
    command: &Option<SyncCommand>,
) -> Result<()> {
    match command {
        Some(SyncCommand::Cache { backend }) => {
            commands::cache::run(commands::cache::CacheOptions {
                backends: list_to_optional_vec(backend),
                verbose: args.global.verbose,
            })
        }
        Some(SyncCommand::Upgrade { backend, no_sync }) => {
            commands::upgrade::run(commands::upgrade::UpgradeOptions {
                backends: list_to_optional_vec(backend),
                no_sync: *no_sync,
                verbose: args.global.verbose,
            })
        }
        Some(SyncCommand::Update {
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        }) => commands::sync::run(build_sync_options(
            args, target, *noconfirm, *hooks, profile, host, modules, *diff, false, true,
        )),
        Some(SyncCommand::Prune {
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        }) => commands::sync::run(build_sync_options(
            args, target, *noconfirm, *hooks, profile, host, modules, *diff, true, false,
        )),
        _ => commands::sync::run(build_sync_options(
            args, target, noconfirm, hooks, profile, host, modules, diff, false, false,
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_sync_options(
    args: &Cli,
    target: &Option<String>,
    noconfirm: bool,
    hooks: bool,
    profile: &Option<String>,
    host: &Option<String>,
    modules: &[String],
    diff: bool,
    prune: bool,
    update: bool,
) -> commands::sync::SyncOptions {
    commands::sync::SyncOptions {
        dry_run: args.global.dry_run,
        prune,
        update,
        verbose: args.global.verbose,
        yes: args.global.yes,
        force: args.global.force,
        target: target.clone(),
        noconfirm,
        hooks,
        profile: profile.clone(),
        host: host.clone(),
        modules: modules.to_vec(),
        diff,
        format: args.global.format.clone(),
        output_version: args.global.output_version.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_info_command(
    args: &Cli,
    query: &Option<String>,
    doctor: bool,
    plan: bool,
    list: bool,
    scope: &Option<InfoListScope>,
    backend: &Option<String>,
    package: &Option<String>,
    profile: &Option<String>,
    host: &Option<String>,
    modules: &[String],
) -> Result<()> {
    let mut mode_count = 0u8;
    if doctor {
        mode_count += 1;
    }
    if plan {
        mode_count += 1;
    }
    if query.is_some() {
        mode_count += 1;
    }
    if list || scope.is_some() {
        mode_count += 1;
    }
    if mode_count > 1 {
        return Err(DeclarchError::Other(
            "Use only one info mode at a time: status, query, --plan, --doctor, or --list [--scope ...]".to_string(),
        ));
    }

    if doctor {
        return commands::info::run(commands::info::InfoOptions {
            doctor: true,
            format: args.global.format.clone(),
            output_version: args.global.output_version.clone(),
            backend: backend.clone(),
            package: package.clone(),
            verbose: args.global.verbose,
        });
    }

    if list || scope.is_some() {
        let (orphans, synced, unmanaged) = match scope {
            Some(InfoListScope::Orphans) => (true, false, false),
            Some(InfoListScope::Synced) => (false, true, false),
            Some(InfoListScope::Unmanaged) => (false, false, true),
            _ => (false, false, false),
        };
        return commands::list::run(commands::list::ListOptions {
            backend: backend.clone(),
            orphans,
            synced,
            unmanaged,
            format: args.global.format.clone(),
            output_version: args.global.output_version.clone(),
        });
    }

    if plan || query.is_some() {
        return commands::info_reason::run(commands::info_reason::InfoReasonOptions {
            query: if plan { None } else { query.clone() },
            target: if plan {
                Some("sync-plan".to_string())
            } else {
                None
            },
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.to_vec(),
            verbose: args.global.verbose,
        });
    }

    commands::info::run(commands::info::InfoOptions {
        doctor: false,
        format: args.global.format.clone(),
        output_version: args.global.output_version.clone(),
        backend: backend.clone(),
        package: package.clone(),
        verbose: args.global.verbose,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_search_command(
    args: &Cli,
    query: &str,
    backends: &[String],
    limit: Option<&str>,
    installed_only: bool,
    available_only: bool,
    local: bool,
) -> Result<()> {
    let parsed_limit = parse_limit_option(limit)?;

    commands::search::run(commands::search::SearchOptions {
        query: query.to_string(),
        backends: list_to_optional_vec(backends),
        limit: parsed_limit,
        installed_only,
        available_only,
        local,
        verbose: args.global.verbose,
        format: args.global.format.clone(),
        output_version: args.global.output_version.clone(),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_lint_command(
    args: &Cli,
    strict: bool,
    fix: bool,
    mode: &LintMode,
    backend: &Option<String>,
    diff: bool,
    benchmark: bool,
    repair_state: bool,
    state_rm: &[String],
    state_rm_backend: &Option<String>,
    state_rm_all: bool,
    profile: &Option<String>,
    host: &Option<String>,
    modules: &[String],
) -> Result<()> {
    commands::lint::run(commands::lint::LintOptions {
        strict,
        fix,
        mode: map_lint_mode(mode),
        backend: backend.clone(),
        diff,
        benchmark,
        repair_state,
        state_rm: state_rm.to_vec(),
        state_rm_backend: state_rm_backend.clone(),
        state_rm_all,
        dry_run: args.global.dry_run,
        yes: args.global.yes,
        format: args.global.format.clone(),
        output_version: args.global.output_version.clone(),
        verbose: args.global.verbose,
        profile: profile.clone(),
        host: host.clone(),
        modules: modules.to_vec(),
    })
}
