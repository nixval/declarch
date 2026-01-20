pub mod config;
pub mod generic;
pub mod parsers;
pub mod registry;
pub mod user_parser;

#[cfg(test)]
mod integration_tests;

pub use config::{BackendConfig, BinarySpecifier, OutputFormat};
pub use generic::GenericManager;

// Re-export commonly used items
pub use registry::get_builtin_backends;
