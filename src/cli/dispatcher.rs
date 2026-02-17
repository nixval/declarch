//! Command dispatcher
//!
//! Routes CLI commands to their appropriate handlers.

use crate::cli::args::{Cli, Command, InfoListScope, LintMode, SyncCommand};
use crate::commands;
use crate::error::{DeclarchError, Result};
use crate::project_identity;
use crate::ui as output;

/// Dispatch the parsed CLI command to the appropriate handler
pub fn dispatch(args: &Cli) -> Result<()> {
    validate_machine_output_contract(args)?;

    match &args.command {
        Some(Command::Init {
            host,
            path,
            backend,
            list,
            local,
            restore_declarch,
        }) => handle_init_command(args, host, path, backend, list, *local, *restore_declarch),

        Some(Command::Sync {
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
            command,
        }) => handle_sync_command(
            args, target, *diff, *noconfirm, *hooks, profile, host, modules, command,
        ),

        Some(Command::Info {
            query,
            doctor,
            plan,
            list,
            scope,
            backend,
            package,
            profile,
            host,
            modules,
        }) => handle_info_command(
            args, query, *doctor, *plan, *list, scope, backend, package, profile, host, modules,
        ),

        Some(Command::Switch {
            old_package,
            new_package,
            backend,
        }) => commands::switch::run(commands::switch::SwitchOptions {
            old_package: old_package.clone(),
            new_package: new_package.clone(),
            backend: backend.clone(),
            dry_run: args.global.dry_run,
            yes: args.global.yes,
            force: args.global.force,
        }),

        Some(Command::Edit {
            target,
            preview,
            number,
            create,
            auto_format,
            validate_only,
            backup,
        }) => commands::edit::run(commands::edit::EditOptions {
            target: target.clone(),
            dry_run: args.global.dry_run,
            preview: *preview,
            number: *number,
            create: *create,
            auto_format: *auto_format,
            validate_only: *validate_only,
            backup: *backup,
        }),

        Some(Command::Install {
            packages,
            backend,
            module,
            no_sync,
        }) => commands::install::run(commands::install::InstallOptions {
            packages: packages.clone(),
            backend: backend.clone(),
            module: module.clone(),
            no_sync: *no_sync,
            yes: args.global.yes,
            dry_run: args.global.dry_run,
            verbose: args.global.verbose,
        }),

        Some(Command::Search {
            query,
            backends,
            limit,
            installed_only,
            available_only,
            local,
        }) => handle_search_command(
            args,
            query,
            backends,
            limit.as_deref(),
            *installed_only,
            *available_only,
            *local,
        ),
        Some(Command::Lint {
            strict,
            fix,
            mode,
            backend,
            diff,
            benchmark,
            repair_state,
            state_rm,
            state_rm_backend,
            state_rm_all,
            profile,
            host,
            modules,
        }) => handle_lint_command(
            args,
            *strict,
            *fix,
            mode,
            backend,
            *diff,
            *benchmark,
            *repair_state,
            state_rm,
            state_rm_backend,
            *state_rm_all,
            profile,
            host,
            modules,
        ),

        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        Some(Command::Ext) => commands::ext::run(),
        Some(Command::SelfUpdate { check, version }) => {
            commands::self_update::run(commands::self_update::SelfUpdateOptions {
                check: *check,
                version: version.clone(),
                yes: args.global.yes,
            })
        }

        None => {
            output::info("No command provided.");
            output::info("Quick start:");
            output::indent(
                &format!(
                    "{} init --backend flatpak soar brew",
                    project_identity::BINARY_NAME
                ),
                2,
            );
            output::indent(
                &format!(
                    "{} edit  // write package you want OR directly",
                    project_identity::BINARY_NAME
                ),
                2,
            );
            output::indent(
                &format!(
                    "{} install flatpak:anypackage",
                    project_identity::BINARY_NAME
                ),
                2,
            );
            output::indent(&format!("{} sync", project_identity::BINARY_NAME), 2);
            output::info(&format!(
                "Use `{} --help` for full command list.",
                project_identity::BINARY_NAME
            ));
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_init_command(
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
fn handle_sync_command(
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
        }) => commands::sync::run(commands::sync::SyncOptions {
            dry_run: args.global.dry_run,
            prune: false,
            update: true,
            verbose: args.global.verbose,
            yes: args.global.yes,
            force: args.global.force,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: *diff,
            format: args.global.format.clone(),
            output_version: args.global.output_version.clone(),
        }),
        Some(SyncCommand::Prune {
            target,
            diff,
            noconfirm,
            hooks,
            profile,
            host,
            modules,
        }) => commands::sync::run(commands::sync::SyncOptions {
            dry_run: args.global.dry_run,
            prune: true,
            update: false,
            verbose: args.global.verbose,
            yes: args.global.yes,
            force: args.global.force,
            target: target.clone(),
            noconfirm: *noconfirm,
            hooks: *hooks,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
            diff: *diff,
            format: args.global.format.clone(),
            output_version: args.global.output_version.clone(),
        }),
        _ => commands::sync::run(commands::sync::SyncOptions {
            dry_run: args.global.dry_run,
            prune: false,
            update: false,
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
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_info_command(
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
fn handle_search_command(
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
fn handle_lint_command(
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

fn map_lint_mode(mode: &LintMode) -> commands::lint::LintMode {
    match mode {
        LintMode::All => commands::lint::LintMode::All,
        LintMode::Validate => commands::lint::LintMode::Validate,
        LintMode::Duplicates => commands::lint::LintMode::Duplicates,
        LintMode::Conflicts => commands::lint::LintMode::Conflicts,
    }
}

fn list_to_optional_vec(values: &[String]) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values.to_vec())
    }
}

fn parse_limit_option(limit: Option<&str>) -> Result<Option<usize>> {
    match limit {
        None => Ok(Some(10)),
        Some("all") | Some("0") => Ok(None),
        Some(raw) => raw.parse::<usize>().map(Some).map_err(|_| {
            DeclarchError::Other(format!(
                "Invalid --limit value '{}'. Use a non-negative integer, 0, or 'all'.",
                raw
            ))
        }),
    }
}

fn validate_machine_output_contract(args: &Cli) -> Result<()> {
    if let Some(version) = args.global.output_version.as_deref() {
        if version != "v1" {
            return Err(DeclarchError::Other(format!(
                "Unsupported output contract version '{}'. Supported: v1",
                version
            )));
        }

        match args.global.format.as_deref() {
            Some("json") | Some("yaml") => {}
            Some(other) => {
                return Err(DeclarchError::Other(format!(
                    "--output-version v1 requires --format json|yaml (got '{}')",
                    other
                )));
            }
            None => {
                output::warning(
                    "--output-version v1 is set without --format; output remains human-oriented.",
                );
            }
        }

        if !supports_v1_contract(args) {
            return Err(DeclarchError::Other(format!(
                "This command does not support --output-version v1 yet.\nSupported now: `{}`, `{}`, `{}`, `{}`, `{}`.",
                project_identity::cli_with("info"),
                project_identity::cli_with("info --list"),
                project_identity::cli_with("lint"),
                project_identity::cli_with("search"),
                project_identity::cli_with("--dry-run sync"),
            )));
        }
    }

    Ok(())
}

fn supports_v1_contract(args: &Cli) -> bool {
    match &args.command {
        Some(Command::Lint { .. }) => true,
        Some(Command::Search { .. }) => true,
        Some(Command::Sync { command: None, .. }) => args.global.dry_run,
        Some(Command::Info {
            doctor,
            plan,
            query,
            ..
        }) => !*doctor && !*plan && query.is_none(),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
