pub mod trait_impl;
pub mod pacman;
pub mod aur;
pub mod flatpak;
pub mod factory;

pub use trait_impl::PackageManager;
pub use factory::PackageManagerFactory;
