use crate::config::kdl_modules::helpers::packages::extract_packages_to;
use crate::config::kdl_modules::parsers::BackendParser;
use crate::config::kdl_modules::types::{Backend, RawConfig};
use crate::error::Result;
use kdl::KdlNode;

/// Generic backend parser for simple backends
///
/// This parser can be used for any backend that follows the standard
/// package extraction pattern without special handling.
///
/// # Examples
/// ```rust,ignore
/// // For npm backend
/// let parser = GenericBackendParser::new(Backend::Npm, "npm", vec![]);
///
/// // For yarn backend with no aliases
/// let parser = GenericBackendParser::new(Backend::Yarn, "yarn", vec![]);
/// ```
pub struct GenericBackendParser {
    backend: Backend,
    name: &'static str,
    aliases: Vec<&'static str>,
}

impl GenericBackendParser {
    /// Create a new generic backend parser
    ///
    /// # Arguments
    /// * `backend` - The Backend enum variant
    /// * `name` - The backend name string (e.g., "npm", "yarn")
    /// * `aliases` - Optional aliases for this backend (e.g., "app" for soar)
    pub fn new(backend: Backend, name: &'static str, aliases: Vec<&'static str>) -> Self {
        Self {
            backend,
            name,
            aliases,
        }
    }
}

impl BackendParser for GenericBackendParser {
    fn name(&self) -> &'static str {
        self.name
    }

    fn aliases(&self) -> &[&'static str] {
        &self.aliases
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, config.packages_for_mut(&self.backend));
        Ok(())
    }
}
