use crate::error::Result;

/// Options for the list command
pub struct ListOptions {
    pub backend: Option<String>,
    pub orphans: bool,
    pub synced: bool,
}

pub fn run(options: ListOptions) -> Result<()> {
    // Phase 1: Basic skeleton
    // TODO: Phase 2 will implement actual filtering logic

    println!("List command");
    println!("Backend filter: {:?}", options.backend);
    println!("Orphans: {}", options.orphans);
    println!("Synced: {}", options.synced);

    Ok(())
}
