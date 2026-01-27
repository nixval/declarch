pub mod critical;
pub mod package_suffixes;
pub mod urls;

pub use critical::ALL as CRITICAL_PACKAGES;
pub use package_suffixes::{VARIANTS, build_variants, is_variant};
pub use urls::{DEFAULT_REGISTRY, RemoteUrlBuilder};
