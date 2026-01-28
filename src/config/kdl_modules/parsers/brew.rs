use crate::config::kdl_modules::types::RawConfig;
use crate::error::Result;
use crate::config::kdl::extract_packages_to;
use kdl::KdlNode;

/// Homebrew backend parser
pub struct BrewParser;

impl super::BackendParser for BrewParser {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.brew_packages);
        Ok(())
    }
}
