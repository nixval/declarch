pub mod package_mappings;
pub mod conflicts;
pub mod env;
pub mod hooks;
pub mod meta;
pub mod packages;
pub mod policy;
pub mod repositories;

// Re-export commonly used functions
pub use packages::{extract_packages_to, extract_mixed_values, extract_mixed_values_return, extract_strings};
pub use meta::get_first_string;
pub use package_mappings::{extract_aliases, parse_alias_entry};
