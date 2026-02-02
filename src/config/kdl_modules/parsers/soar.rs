use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// Soar (static binaries) backend parser
pub struct SoarParser;

impl super::BackendParser for SoarParser {
    fn name(&self) -> &'static str {
        "soar"
    }

    fn aliases(&self) -> &[&'static str] {
        &["app"] // "app" is an alias for "soar"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&Backend::Soar));
        Ok(())
    }
}
