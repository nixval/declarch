use crate::core::resolver;
use crate::state::types::State;
use crate::ui as output;
use chrono::Utc;
use colored::Colorize;

use super::SyncOptions;

pub(super) fn warn_partial_upgrade_impl(
    state: &State,
    tx: &resolver::Transaction,
    options: &SyncOptions,
) {
    if !options.update && !tx.to_install.is_empty() {
        let should_warn = match state.meta.last_update {
            Some(last) => Utc::now().signed_duration_since(last).num_hours() > 24,
            None => true,
        };

        if should_warn {
            let time_str = state
                .meta
                .last_update
                .map(|t| format!("{}h ago", Utc::now().signed_duration_since(t).num_hours()))
                .unwrap_or("unknown".to_string());

            output::separator();
            println!(
                "{} Last system update: {}. Use {} to refresh.",
                "⚠ Partial Upgrade Risk:".yellow().bold(),
                time_str.white(),
                "--update".bold()
            );
        }
    }
}
