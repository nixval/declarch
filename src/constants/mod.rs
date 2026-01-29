pub mod common;
pub mod critical;
pub mod package_suffixes;
pub mod urls;

pub use common::{
    CONFIG_DIR_NAME, CONFIG_EXTENSION, CONFIG_FILE_NAME, DECLARCH_DIR_NAME,
    DEFAULT_BRANCHES, MODULES_DIR_NAME, PROJECT_NAME, PROJECT_ORG, PROJECT_QUALIFIER,
    BACKENDS_FILE_NAME, STATE_FILE_NAME,
};
pub use critical::ALL as CRITICAL_PACKAGES;
pub use package_suffixes::{VARIANTS, build_variants, is_variant};
pub use urls::{DEFAULT_REGISTRY, RemoteUrlBuilder};
