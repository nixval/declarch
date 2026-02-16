// Common constants used throughout the codebase

/// Configuration file extension
pub const CONFIG_EXTENSION: &str = "kdl";

/// Project name
pub const PROJECT_NAME: &str = "declarch";

/// Project organization (reverse domain notation)
pub const PROJECT_QUALIFIER: &str = "com";
pub const PROJECT_ORG: &str = "declarch";

/// Default git branch names to check
pub const DEFAULT_BRANCHES: &[&str] = &["main", "master"];

/// Default configuration directory name
pub const CONFIG_DIR_NAME: &str = ".config";

/// Declarch configuration directory name
pub const DECLARCH_DIR_NAME: &str = "declarch";

/// Default configuration file name
pub const CONFIG_FILE_NAME: &str = "declarch.kdl";

/// Modules directory name
pub const MODULES_DIR_NAME: &str = "modules";

/// State file name
pub const STATE_FILE_NAME: &str = "state.json";

/// Backends configuration file name
pub const BACKENDS_FILE_NAME: &str = "backends.kdl";

/// Default timeout (seconds) for backend command execution.
pub const BACKEND_COMMAND_TIMEOUT_SECS: u64 = 300;

/// Timeout (seconds) for search result collection window per backend.
pub const SEARCH_BACKEND_TIMEOUT_SECS: u64 = 30;

/// Timeout (seconds) for hook execution.
pub const HOOK_TIMEOUT_SECS: u64 = 30;

/// Maximum retry attempts for failed backend mutating operations.
pub const BACKEND_OPERATION_MAX_RETRIES: u32 = 3;

/// Delay between retries in milliseconds.
pub const BACKEND_RETRY_DELAY_MS: u64 = 1000;
