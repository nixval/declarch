//! Command dispatcher
//!
//! Routes CLI commands to their appropriate handlers.

mod normalization;
mod output_contract;
mod routing;

use crate::cli::args::{Cli, Command};
use crate::commands;
use crate::error::Result;
use crate::project_identity;
use crate::ui as output;
use output_contract::validate_machine_output_contract;
use routing::{
    handle_info_command, handle_init_command, handle_lint_command, handle_search_command,
    handle_sync_command,
};

/// Dispatch the parsed CLI command to the appropriate handler.
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
        None => print_no_command_help(),
    }
}

fn print_no_command_help() -> Result<()> {
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

#[cfg(test)]
mod tests;
