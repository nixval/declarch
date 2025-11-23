use crate::package::trait_impl::PackageManager;
use crate::package::{aur::AurManager, flatpak::FlatpakManager};
use crate::config::types::AurHelper;
use crate::state::types::Backend;
use crate::utils::errors::Result;

pub struct PackageManagerFactory;

impl PackageManagerFactory {
    pub fn get(backend: Backend, aur_helper: AurHelper) -> Result<Box<dyn PackageManager>> {
        match backend {
            Backend::Aur => Ok(Box::new(AurManager { helper: aur_helper })),
            Backend::Flatpak => Ok(Box::new(FlatpakManager)),
        }
    }
}
