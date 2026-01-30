use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use kdl::KdlNode;

/// Flatpak backend parser
pub struct FlatpakParser;

impl super::BackendParser for FlatpakParser {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.flatpak_packages);
        Ok(())
    }
}
