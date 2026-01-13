pub mod traits;
pub mod aur;
pub mod flatpak;
pub mod soar;
pub mod registry;

pub use traits::PackageManager;
pub use registry::{BackendRegistry, get_registry, create_manager};
