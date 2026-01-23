pub mod aur;
pub mod flatpak;
pub mod registry;
pub mod soar;
pub mod traits;

pub use registry::{BackendRegistry, create_manager, get_registry};
pub use traits::PackageManager;
