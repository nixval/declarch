use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// Flatpak backend parser
pub struct FlatpakParser;

impl super::BackendParser for FlatpakParser {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&Backend::Flatpak));
        Ok(())
    }
}
