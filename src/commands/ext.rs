use crate::error::Result;
use crate::ui as output;

/// Hidden placeholder for future extension protocol.
pub fn run() -> Result<()> {
    output::header("Extension Protocol Placeholder");
    output::info("External extension runtime is not implemented yet.");
    output::info("Planned discovery pattern: declarch-ext-*");
    output::info("Planned contract version: v1");
    output::info("See: docs/contracts/v1/README.md");
    Ok(())
}
