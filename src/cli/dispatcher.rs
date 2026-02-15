//! Command dispatcher
//!
//! Routes CLI commands to their appropriate handlers and manages
//! deprecated flag handling.

use crate::cli::args::{Cli, Command, LintMode, SyncCommand};
use crate::commands;
use crate::error::{DeclarchError, Result};
use crate::ui as output;

use super::deprecated::{
    handle_deprecated_sync_flags, show_deprecation_warning, sync_command_to_options,
};

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
            restore_backends,
            restore_declarch,
        }) => {
            // Handle --list flag first
            if let Some(what) = list {
                if what == "backends" {
                    return commands::init::list_available_backends();
                } else if what == "modules" {
                    return commands::init::list_available_modules();
                } else {
                    return Err(DeclarchError::Other(format!(
                        "Unknown list target: '{}'. Available: backends, modules",
                        what
                    )));
                }
            }

            // Handle restore flags
            if *restore_backends {
                return commands::init::restore_backends();
            }
            if *restore_declarch {
                return commands::init::restore_declarch(host.clone());
            }

            commands::init::run(commands::init::InitOptions {
                host: host.clone(),
                path: path.clone(),
                backends: backend.clone(),
                force: args.global.force,
                yes: args.global.yes,
                local: *local,
            })
        }

        Some(Command::Sync { command, gc }) => {
            match command {
                Some(SyncCommand::Cache { backend }) => {
                    commands::cache::run(commands::cache::CacheOptions {
                        backends: if backend.is_empty() {
                            None
                        } else {
                            Some(backend.clone())
                        },
                        verbose: args.global.verbose,
                    })
                }
                Some(SyncCommand::Upgrade { backend, no_sync }) => {
                    commands::upgrade::run(commands::upgrade::UpgradeOptions {
                        backends: if backend.is_empty() {
                            None
                        } else {
                            Some(backend.clone())
                        },
                        no_sync: *no_sync,
                        verbose: args.global.verbose,
                    })
                }
                _ => {
                    // Handle other sync subcommands (Sync, Preview, Update, Prune)
                    let (has_deprecated_flags, deprecated_command, new_cmd_str) =
                        handle_deprecated_sync_flags(false, false, false, *gc);

                    // Use the command from subcommand if provided, otherwise use deprecated flags
                    let sync_cmd = command
                        .clone()
                        .unwrap_or_else(|| deprecated_command.clone());

                    // Show deprecation warning if old flags were used
                    if has_deprecated_flags {
                        show_deprecation_warning(new_cmd_str);
                    }

                    // Convert and execute
                    let mut options =
                        sync_command_to_options(&sync_cmd, args.global.yes, args.global.force);
                    options.format = args.global.format.clone();
                    options.output_version = args.global.output_version.clone();
                    commands::sync::run(options)
                }
            }
        }

        Some(Command::Info {
            query,
            doctor,
            plan,
            list,
            orphans,
            synced,
            backend,
            package,
            profile,
            host,
            modules,
        }) => {
            let mut mode_count = 0u8;
            if *doctor {
                mode_count += 1;
            }
            if *plan {
                mode_count += 1;
            }
            if query.is_some() {
                mode_count += 1;
            }
            if *list || *orphans || *synced {
                mode_count += 1;
            }
            if mode_count > 1 {
                return Err(DeclarchError::Other(
                    "Use only one info mode at a time: status, query, --plan, --doctor, or --list/--orphans/--synced".to_string(),
                ));
            }

            if *doctor {
                return commands::info::run(commands::info::InfoOptions {
                    doctor: true,
                    format: args.global.format.clone(),
                    output_version: args.global.output_version.clone(),
                    backend: backend.clone(),
                    package: package.clone(),
                    verbose: args.global.verbose,
                });
            }

            if *list || *orphans || *synced {
                return commands::list::run(commands::list::ListOptions {
                    backend: backend.clone(),
                    orphans: *orphans,
                    synced: *synced,
                    format: args.global.format.clone(),
                    output_version: args.global.output_version.clone(),
                });
            }

            if *plan || query.is_some() {
                return commands::info_reason::run(commands::info_reason::InfoReasonOptions {
                    query: if *plan { None } else { query.clone() },
                    target: if *plan {
                        Some("sync-plan".to_string())
                    } else {
                        None
                    },
                    profile: profile.clone(),
                    host: host.clone(),
                    modules: modules.clone(),
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

        Some(Command::Switch {
            old_package,
            new_package,
            backend,
            dry_run,
        }) => commands::switch::run(commands::switch::SwitchOptions {
            old_package: old_package.clone(),
            new_package: new_package.clone(),
            backend: backend.clone(),
            dry_run: *dry_run,
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
        }),

        Some(Command::Search {
            query,
            backends,
            limit,
            installed_only,
            available_only,
            local,
        }) => {
            let parsed_limit = parse_limit_option(limit.as_deref())?;

            commands::search::run(commands::search::SearchOptions {
                query: query.clone(),
                backends: if backends.is_empty() {
                    None
                } else {
                    Some(backends.clone())
                },
                limit: parsed_limit,
                installed_only: *installed_only,
                available_only: *available_only,
                local: *local,
                format: args.global.format.clone(),
                output_version: args.global.output_version.clone(),
            })
        }
        Some(Command::Lint {
            strict,
            fix,
            mode,
            backend,
            diff,
            benchmark,
            repair_state,
            profile,
            host,
            modules,
        }) => commands::lint::run(commands::lint::LintOptions {
            strict: *strict,
            fix: *fix,
            mode: match mode {
                LintMode::All => commands::lint::LintMode::All,
                LintMode::Validate => commands::lint::LintMode::Validate,
                LintMode::Duplicates => commands::lint::LintMode::Duplicates,
                LintMode::Conflicts => commands::lint::LintMode::Conflicts,
            },
            backend: backend.clone(),
            diff: *diff,
            benchmark: *benchmark,
            repair_state: *repair_state,
            format: args.global.format.clone(),
            output_version: args.global.output_version.clone(),
            verbose: args.global.verbose,
            profile: profile.clone(),
            host: host.clone(),
            modules: modules.clone(),
        }),

        Some(Command::Completions { shell }) => commands::completions::run(*shell),
        Some(Command::Ext) => commands::ext::run(),

        None => {
            output::info("No command provided. Use --help.");
            Ok(())
        }
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
            return Err(DeclarchError::Other(
                "This command does not support --output-version v1 yet. Supported now: `info`, `info --list`, `lint`, `search`, `sync preview`.".to_string(),
            ));
        }
    }

    Ok(())
}

fn supports_v1_contract(args: &Cli) -> bool {
    match &args.command {
        Some(Command::Lint { .. }) => true,
        Some(Command::Search { .. }) => true,
        Some(Command::Sync {
            command: Some(SyncCommand::Preview { .. }),
            ..
        }) => true,
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
mod tests {
    use super::{parse_limit_option, validate_machine_output_contract};
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
            command: Some(SyncCommand::Update {
                gc: false,
                target: None,
                diff: false,
                noconfirm: false,
                hooks: false,
                profile: None,
                host: None,
                modules: Vec::new(),
            }),
            gc: false,
        });
        assert!(validate_machine_output_contract(&cli).is_err());
    }

    #[test]
    fn output_version_allows_sync_preview() {
        use crate::cli::args::{Command, SyncCommand};
        let mut cli = base_cli();
        cli.global.output_version = Some("v1".to_string());
        cli.global.format = Some("json".to_string());
        cli.command = Some(Command::Sync {
            command: Some(SyncCommand::Preview {
                gc: false,
                target: None,
                noconfirm: false,
                hooks: false,
                profile: None,
                host: None,
                modules: Vec::new(),
            }),
            gc: false,
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
}
