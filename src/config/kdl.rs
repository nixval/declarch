use crate::error::Result;
use kdl::{KdlDocument, KdlNode};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct RawConfig {
    // === Existing fields ===
    pub imports: Vec<String>,
    /// Packages from AUR (Arch Linux specific)
    /// Syntax: packages { ... } or packages:aur { ... }
    pub packages: Vec<PackageEntry>,
    /// Packages from Soar registry (cross-distro static binaries)
    /// Syntax: packages:soar { ... } or soar:package in packages block
    pub soar_packages: Vec<PackageEntry>,
    /// Flatpak packages
    /// Syntax: packages:flatpak { ... } or flatpak:package in packages block
    pub flatpak_packages: Vec<PackageEntry>,

    // === New language ecosystem backends ===
    /// npm packages (Node.js global packages)
    /// Syntax: packages:npm { ... } or npm:package in packages block
    pub npm_packages: Vec<PackageEntry>,
    /// Yarn packages
    /// Syntax: packages:yarn { ... }
    pub yarn_packages: Vec<PackageEntry>,
    /// pnpm packages
    /// Syntax: packages:pnpm { ... }
    pub pnpm_packages: Vec<PackageEntry>,
    /// Bun packages
    /// Syntax: packages:bun { ... }
    pub bun_packages: Vec<PackageEntry>,
    /// pip packages (Python)
    /// Syntax: packages:pip { ... }
    pub pip_packages: Vec<PackageEntry>,
    /// Cargo packages (Rust)
    /// Syntax: packages:cargo { ... }
    pub cargo_packages: Vec<PackageEntry>,
    /// Homebrew packages
    /// Syntax: packages:brew { ... }
    pub brew_packages: Vec<PackageEntry>,

    /// Custom backend packages (user-defined)
    /// Syntax: packages:nala { ... } where nala is defined in backends.kdl
    /// Stored as HashMap: backend_name -> Vec<PackageEntry>
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,

    pub excludes: Vec<String>,
    /// Package aliases: config_name -> actual_package_name
    /// Example: "pipewire" -> "pipewire-jack2"
    pub aliases: HashMap<String, String>,
    /// Editor to use for edit command
    /// Syntax: editor "nvim" or editor nvim
    pub editor: Option<String>,

    // === NEW: Meta block ===
    /// Configuration metadata
    pub meta: ConfigMeta,

    // === NEW: Conflicts ===
    /// Mutually exclusive packages
    pub conflicts: Vec<ConflictEntry>,

    // === NEW: Backend options ===
    /// Backend-specific configuration options
    /// Syntax: options:aur { noconfirm true }
    pub backend_options: HashMap<String, HashMap<String, String>>,

    // === NEW: Environment variables ===
    /// Environment variables for package operations
    /// Syntax: env { "EDITOR=nvim" } or env:aur { "MAKEFLAGS=-j4" }
    pub env: HashMap<String, Vec<String>>,

    // === NEW: Package repositories ===
    /// Custom package repositories
    /// Syntax: repos:aur { "https://..." }
    pub repositories: HashMap<String, Vec<String>>,

    // === NEW: Policy control ===
    /// Package lifecycle policies
    pub policy: PolicyConfig,

    // === NEW: Hooks ===
    /// Pre/post sync hooks
    pub hooks: HookConfig,
}

/// Package entry (version constraints skipped for now)
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
}

/// Configuration metadata
#[derive(Debug, Clone, Default)]
pub struct ConfigMeta {
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

/// Conflict entry - mutually exclusive packages
#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub packages: Vec<String>,
    pub condition: Option<String>, // Future: for conditional conflicts
}

/// Package lifecycle policies
#[derive(Debug, Clone, Default)]
pub struct PolicyConfig {
    /// Protected packages that won't be removed even with --prune
    pub protected: HashSet<String>,
    /// Strategy for handling orphans: "keep", "remove", "ask"
    pub orphans: Option<String>,
}

/// Hook configuration
#[derive(Debug, Clone, Default)]
pub struct HookConfig {
    /// Pre-sync hooks
    pub pre_sync: Vec<HookEntry>,
    /// Post-sync hooks
    pub post_sync: Vec<HookEntry>,
}

/// Hook entry
#[derive(Debug, Clone)]
pub struct HookEntry {
    pub command: String,
    pub hook_type: HookType,
}

/// Hook type
#[derive(Debug, Clone, PartialEq)]
pub enum HookType {
    Run,        // Run without sudo
    SudoNeeded, // Explicitly needs sudo
    Script,     // Run a script file
    Backup,     // Backup a file
    Notify,     // Send notification
}

/// Trait for backend-specific package parsing
///
/// Each backend (AUR, Soar, Flatpak) implements this trait
/// to define how it parses packages from KDL nodes.
pub trait BackendParser: Send + Sync {
    /// Backend identifier (e.g., "aur", "soar", "flatpak")
    fn name(&self) -> &'static str;

    /// Aliases for this backend (e.g., "app" is an alias for "soar")
    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    /// Parse packages from a KDL node and add them to the config
    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()>;

    /// Check if a backend name matches this parser (including aliases)
    fn matches(&self, backend: &str) -> bool {
        self.name() == backend || self.aliases().contains(&backend)
    }
}

/// AUR (Arch User Repository) backend parser
struct AurParser;

impl BackendParser for AurParser {
    fn name(&self) -> &'static str {
        "aur"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.packages);
        Ok(())
    }
}

/// Soar (static binaries) backend parser
struct SoarParser;

impl BackendParser for SoarParser {
    fn name(&self) -> &'static str {
        "soar"
    }

    fn aliases(&self) -> &[&'static str] {
        &["app"] // "app" is an alias for "soar"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.soar_packages);
        Ok(())
    }
}

/// Flatpak backend parser
struct FlatpakParser;

impl BackendParser for FlatpakParser {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.flatpak_packages);
        Ok(())
    }
}

/// npm backend parser
struct NpmParser;

impl BackendParser for NpmParser {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.npm_packages);
        Ok(())
    }
}

/// Yarn backend parser
struct YarnParser;

impl BackendParser for YarnParser {
    fn name(&self) -> &'static str {
        "yarn"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.yarn_packages);
        Ok(())
    }
}

/// pnpm backend parser
struct PnpmParser;

impl BackendParser for PnpmParser {
    fn name(&self) -> &'static str {
        "pnpm"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.pnpm_packages);
        Ok(())
    }
}

/// Bun backend parser
struct BunParser;

impl BackendParser for BunParser {
    fn name(&self) -> &'static str {
        "bun"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.bun_packages);
        Ok(())
    }
}

/// pip backend parser
struct PipParser;

impl BackendParser for PipParser {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.pip_packages);
        Ok(())
    }
}

/// Cargo backend parser
struct CargoParser;

impl BackendParser for CargoParser {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.cargo_packages);
        Ok(())
    }
}

/// Homebrew backend parser
struct BrewParser;

impl BackendParser for BrewParser {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        extract_packages_to(node, &mut config.brew_packages);
        Ok(())
    }
}

/// Registry for backend parsers
///
/// This registry manages all available backend parsers and provides
/// a unified interface for parsing packages from KDL nodes.
struct BackendParserRegistry {
    parsers: Vec<Box<dyn BackendParser>>,
    #[allow(dead_code)]
    default_backend: &'static str, // Reserved for future use
}

impl BackendParserRegistry {
    /// Create a new registry with default parsers
    fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(AurParser),
                Box::new(SoarParser),
                Box::new(FlatpakParser),
                Box::new(NpmParser),
                Box::new(YarnParser),
                Box::new(PnpmParser),
                Box::new(BunParser),
                Box::new(PipParser),
                Box::new(CargoParser),
                Box::new(BrewParser),
            ],
            default_backend: "aur", // Default to AUR for Arch Linux
        }
    }

    /// Find a parser by backend name (including aliases)
    fn find_parser(&self, backend: &str) -> Option<&dyn BackendParser> {
        self.parsers
            .iter()
            .find(|p| p.matches(backend))
            .map(|p| p.as_ref())
    }

    /// Parse packages with inline prefix syntax
    ///
    /// Handles syntax like: `packages { aur:hyprland soar:bat }`
    fn parse_inline_prefix(&self, package_str: &str, config: &mut RawConfig) -> Result<()> {
        if let Some((backend, package)) = package_str.split_once(':') {
            if self.find_parser(backend).is_some() {
                let entry = PackageEntry {
                    name: package.to_string(),
                };

                // Directly add to the appropriate config vector based on backend
                match backend {
                    "aur" => config.packages.push(entry),
                    "soar" | "app" => config.soar_packages.push(entry),
                    "flatpak" => config.flatpak_packages.push(entry),
                    "npm" => config.npm_packages.push(entry),
                    "yarn" => config.yarn_packages.push(entry),
                    "pnpm" => config.pnpm_packages.push(entry),
                    "bun" => config.bun_packages.push(entry),
                    "pip" => config.pip_packages.push(entry),
                    "cargo" => config.cargo_packages.push(entry),
                    "brew" => config.brew_packages.push(entry),
                    _ => config.packages.push(entry),
                }
            } else {
                // Unknown backend - treat the whole string as package name with default backend
                config.packages.push(PackageEntry {
                    name: package_str.to_string(),
                });
            }
        } else {
            // No prefix - use default backend (AUR)
            config.packages.push(PackageEntry {
                name: package_str.to_string(),
            });
        }
        Ok(())
    }

    /// Parse a packages node with flexible syntax
    ///
    /// Supported syntaxes:
    /// 1. `packages { hyprland waybar }` → AUR packages (default)
    /// 2. `packages:aur { hyprland }` → AUR packages (explicit)
    /// 3. `packages:soar { bat exa }` → Soar packages
    /// 4. `packages:flatpak { com.spotify.Client }` → Flatpak packages
    /// 5. `packages { bat aur:hyprland flatpak:app.id }` → Mixed with inline prefix
    /// 6. `packages { soar { bat } flatpak { app.id } }` → Nested blocks
    fn parse_packages_node(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        let node_name = node.name().value();

        // Case 1: Colon syntax: packages:backend
        if let Some((_, backend)) = node_name.split_once(':') {
            if let Some(parser) = self.find_parser(backend) {
                return parser.parse(node, config);
            }
            // Custom backend - parse and store in custom_packages HashMap
            return self.parse_custom_backend(backend, node, config);
        }

        // Case 2: Check for nested children (backend blocks)
        if let Some(children) = node.children() {
            for child in children.nodes() {
                let child_name = child.name().value();

                // Check if child name is a backend identifier
                if let Some(parser) = self.find_parser(child_name) {
                    // Parse as backend block: `packages { aur { ... } }`
                    parser.parse(child, config)?;
                } else {
                    // Check for inline prefix syntax: `aur:hyprland`
                    if child_name.contains(':') {
                        self.parse_inline_prefix(child_name, config)?;
                    } else {
                        // No backend prefix - use default backend
                        config.packages.push(PackageEntry {
                            name: child_name.to_string(),
                        });
                    }

                    // Also check for string arguments in the child node
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            if val.contains(':') {
                                self.parse_inline_prefix(val, config)?;
                            } else {
                                config.packages.push(PackageEntry {
                                    name: val.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Case 3: Extract direct string arguments (default to AUR)
        for entry in node.entries() {
            if let Some(val) = entry.value().as_string() {
                if val.contains(':') {
                    self.parse_inline_prefix(val, config)?;
                } else {
                    config.packages.push(PackageEntry {
                        name: val.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Parse packages for a custom (user-defined) backend
    fn parse_custom_backend(
        &self,
        backend_name: &str,
        node: &KdlNode,
        config: &mut RawConfig,
    ) -> Result<()> {
        let mut packages = Vec::new();

        // Extract packages from children
        if let Some(children) = node.children() {
            for child in children.nodes() {
                let child_name = child.name().value();
                packages.push(PackageEntry {
                    name: child_name.to_string(),
                });

                // Also check for string arguments
                for entry in child.entries() {
                    if let Some(val) = entry.value().as_string() {
                        packages.push(PackageEntry {
                            name: val.to_string(),
                        });
                    }
                }
            }
        }

        // Extract packages from direct arguments
        for entry in node.entries() {
            if let Some(val) = entry.value().as_string() {
                packages.push(PackageEntry {
                    name: val.to_string(),
                });
            }
        }

        // Store in custom_packages HashMap
        config
            .custom_packages
            .insert(backend_name.to_string(), packages);

        Ok(())
    }
}

impl Default for BackendParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_kdl_content(content: &str) -> Result<RawConfig> {
    let doc: KdlDocument = content.parse().map_err(|e: kdl::KdlError| {
        // Provide more helpful error messages for common KDL syntax issues
        let err_msg = e.to_string();
        let hint = if err_msg.contains("unexpected token") {
            "\nHint: Check for missing quotes, unmatched brackets, or invalid characters."
        } else if err_msg.contains("unexpected end of file") {
            "\nHint: You might be missing a closing brace '}' or parenthesis."
        } else if err_msg.contains("expected") {
            "\nHint: Check that your KDL syntax follows the format: node-name \"value\" { ... }"
        } else {
            ""
        };

        crate::error::DeclarchError::ConfigError(format!("KDL parsing error: {}{}", err_msg, hint))
    })?;

    let mut config = RawConfig {
        imports: vec![],
        packages: vec![],
        soar_packages: vec![],
        flatpak_packages: vec![],
        npm_packages: vec![],
        yarn_packages: vec![],
        pnpm_packages: vec![],
        bun_packages: vec![],
        pip_packages: vec![],
        cargo_packages: vec![],
        brew_packages: vec![],
        custom_packages: HashMap::new(),
        excludes: vec![],
        aliases: HashMap::new(),
        editor: None,
        meta: ConfigMeta::default(),
        conflicts: vec![],
        backend_options: HashMap::new(),
        env: HashMap::new(),
        repositories: HashMap::new(),
        policy: PolicyConfig::default(),
        hooks: HookConfig::default(),
    };

    let registry = BackendParserRegistry::new();

    for node in doc.nodes() {
        let node_name = node.name().value();

        match node_name {
            "import" | "imports" => {
                extract_strings(node, &mut config.imports);
            }
            "exclude" | "excludes" => {
                extract_mixed_values(node, &mut config.excludes);
            }
            "aliases-pkg" | "alias-pkg" => {
                extract_aliases(node, &mut config.aliases);
            }
            "editor" => {
                // Extract editor from first string argument
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.editor = Some(val.to_string());
                }
            }
            "description" => {
                // Parse description into meta
                if let Some(entry) = node.entries().first()
                    && let Some(val) = entry.value().as_string()
                {
                    config.meta.description = Some(val.to_string());
                }
            }
            // NEW: Meta block
            "meta" => {
                parse_meta_block(node, &mut config.meta)?;
            }
            // NEW: Conflicts
            "conflicts" | "conflict" => {
                parse_conflicts(node, &mut config.conflicts)?;
            }
            // NEW: Backend options
            name if name.starts_with("options") => {
                parse_backend_options(node, &mut config.backend_options)?;
            }
            // NEW: Environment variables
            name if name.starts_with("env") => {
                parse_env_vars(node, &mut config.env, None)?;
            }
            // NEW: Package repositories
            name if name.starts_with("repos") || name.starts_with("repositories") => {
                parse_repositories(node, &mut config.repositories)?;
            }
            // NEW: Policy
            "policy" => {
                parse_policy(node, &mut config.policy)?;
            }
            // NEW: Hooks
            "hooks" => {
                parse_hooks(node, &mut config.hooks)?;
            }
            // NEW: Simplified flat hooks
            "on-sync" => {
                if let Some(val) = get_first_string(node) {
                    config.hooks.post_sync.push(HookEntry {
                        command: val,
                        hook_type: HookType::Run,
                    });
                }
            }
            "on-sync-sudo" => {
                if let Some(val) = get_first_string(node) {
                    config.hooks.post_sync.push(HookEntry {
                        command: val,
                        hook_type: HookType::SudoNeeded,
                    });
                }
            }
            "on-pre-sync" => {
                if let Some(val) = get_first_string(node) {
                    config.hooks.pre_sync.push(HookEntry {
                        command: val,
                        hook_type: HookType::Run,
                    });
                }
            }
            // Parse packages with flexible syntax using the registry
            name if name.starts_with("packages") => {
                registry.parse_packages_node(node, &mut config)?;
            }
            // Legacy syntax support (with deprecation warning in the future)
            "aur-packages" | "aur-package" => {
                let packages = extract_mixed_values_return(node);
                config
                    .packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "soar-packages" | "soar-package" => {
                let packages = extract_mixed_values_return(node);
                config
                    .soar_packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            "flatpak-packages" | "flatpak-package" => {
                let packages = extract_mixed_values_return(node);
                config
                    .flatpak_packages
                    .extend(packages.into_iter().map(|p| PackageEntry { name: p }));
            }
            _ => {}
        }
    }

    Ok(config)
}

/// Parse meta block: meta { description "..." author "..." version "..." }
fn parse_meta_block(node: &KdlNode, meta: &mut ConfigMeta) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            match child_name {
                "description" => {
                    if let Some(val) = get_first_string(child) {
                        meta.description = Some(val);
                    }
                }
                "author" => {
                    if let Some(val) = get_first_string(child) {
                        meta.author = Some(val);
                    }
                }
                "version" => {
                    if let Some(val) = get_first_string(child) {
                        meta.version = Some(val);
                    }
                }
                "tags" => {
                    // tags can be multiple: tags ["workstation" "gaming"]
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            meta.tags.push(val.to_string());
                        }
                    }
                    if let Some(children) = child.children() {
                        for tag_child in children.nodes() {
                            meta.tags.push(tag_child.name().value().to_string());
                        }
                    }
                }
                "url" => {
                    if let Some(val) = get_first_string(child) {
                        meta.url = Some(val);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Parse conflicts block: conflicts { vim neovim }
fn parse_conflicts(node: &KdlNode, conflicts: &mut Vec<ConflictEntry>) -> Result<()> {
    let mut packages = Vec::new();

    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            packages.push(val.to_string());
        }
    }

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            packages.push(child.name().value().to_string());
            // Also check for string arguments in child entries
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    packages.push(val.to_string());
                }
            }
        }
    }

    if !packages.is_empty() {
        conflicts.push(ConflictEntry {
            packages,
            condition: None,
        });
    }

    Ok(())
}

/// Parse backend options: options:aur { noconfirm true }
fn parse_backend_options(
    node: &KdlNode,
    options: &mut HashMap<String, HashMap<String, String>>,
) -> Result<()> {
    // Check for colon syntax: options:aur
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified, apply to all? Or skip?
        // For now, skip if no backend specified
        return Ok(());
    };

    let mut opts = HashMap::new();

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let key = child.name().value();
            if let Some(val) = get_first_string(child) {
                opts.insert(key.to_string(), val);
            } else if let Some(val) = child.entries().first()
                && let Some(val) = val.value().as_string()
            {
                opts.insert(key.to_string(), val.to_string());
            } else {
                // Boolean flag without value
                opts.insert(key.to_string(), "true".to_string());
            }
        }
    }

    // Extract from string arguments (key=value format)
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string()
            && let Some((key, v)) = val.split_once('=')
        {
            opts.insert(key.to_string(), v.to_string());
        }
    }

    if !opts.is_empty() {
        options.insert(backend_name, opts);
    }

    Ok(())
}

/// Parse environment variables: env { "EDITOR=nvim" } or env:aur { "MAKEFLAGS=-j4" }
fn parse_env_vars(
    node: &KdlNode,
    env: &mut HashMap<String, Vec<String>>,
    backend: Option<&str>,
) -> Result<()> {
    // Check for colon syntax: env:aur
    let backend_name = if let Some((_, b)) = node.name().value().split_once(':') {
        b.to_string()
    } else {
        backend.unwrap_or("global").to_string()
    };

    let mut vars = Vec::new();

    // Extract from string arguments (format: "EDITOR=nvim")
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            vars.push(val.to_string());
        }
    }

    // Extract from named arguments (format: EDITOR="nvim")
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            let key = name.value();
            if let Some(val) = entry.value().as_string() {
                // Only format as key=value if not already in key=value format
                if !key.is_empty()
                    && !key.contains('(')
                    && !vars.contains(&format!("{}={}", key, val))
                {
                    vars.push(format!("{}={}", key, val));
                }
            }
        }
    }

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            // Child name could be the key, value in arguments
            let key = child.name().value();
            if let Some(val) = get_first_string(child) {
                vars.push(format!("{}={}", key, val));
            } else if let Some(val) = child.entries().first()
                && let Some(val) = val.value().as_string()
            {
                vars.push(format!("{}={}", key, val));
            }
        }
    }

    if !vars.is_empty() {
        env.insert(backend_name, vars);
    }

    Ok(())
}

/// Parse repositories: repos:aur { "https://..." }
fn parse_repositories(node: &KdlNode, repos: &mut HashMap<String, Vec<String>>) -> Result<()> {
    // Check for colon syntax: repos:aur
    let backend_name = if let Some((_, backend)) = node.name().value().split_once(':') {
        backend.to_string()
    } else {
        // No backend specified - skip
        return Ok(());
    };

    let mut repo_urls = Vec::new();

    // Extract from string arguments
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            repo_urls.push(val.to_string());
        }
    }

    // Extract from children
    if let Some(children) = node.children() {
        for child in children.nodes() {
            repo_urls.push(child.name().value().to_string());
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    repo_urls.push(val.to_string());
                }
            }
        }
    }

    if !repo_urls.is_empty() {
        repos.insert(backend_name, repo_urls);
    }

    Ok(())
}

/// Parse policy block: policy { protected { linux systemd } orphans "keep" }
fn parse_policy(node: &KdlNode, policy: &mut PolicyConfig) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "protected" => {
                    // Extract protected packages
                    for entry in child.entries() {
                        if let Some(val) = entry.value().as_string() {
                            policy.protected.insert(val.to_string());
                        }
                    }
                    if let Some(grandchildren) = child.children() {
                        for gc in grandchildren.nodes() {
                            policy.protected.insert(gc.name().value().to_string());
                        }
                    }
                }
                "orphans" => {
                    // Extract orphans strategy
                    if let Some(val) = get_first_string(child) {
                        policy.orphans = Some(val);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Parse hooks block: hooks { post-sync { sudo-needed "systemctl restart gdm" } }
fn parse_hooks(node: &KdlNode, hooks: &mut HookConfig) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "pre-sync" => {
                    parse_hook_entries(child, &mut hooks.pre_sync)?;
                }
                "post-sync" => {
                    parse_hook_entries(child, &mut hooks.post_sync)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Parse individual hook entries
fn parse_hook_entries(node: &KdlNode, entries: &mut Vec<HookEntry>) -> Result<()> {
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();

            let hook_type = match child_name {
                "run" => HookType::Run,
                "sudo-needed" => HookType::SudoNeeded,
                "script" => HookType::Script,
                "backup" => HookType::Backup,
                "notify" => HookType::Notify,
                _ => HookType::Run, // Default to Run
            };

            if let Some(command) = get_first_string(child) {
                entries.push(HookEntry {
                    command: command.to_string(),
                    hook_type,
                });
            } else if let Some(entry) = child.entries().first()
                && let Some(val) = entry.value().as_string()
            {
                entries.push(HookEntry {
                    command: val.to_string(),
                    hook_type,
                });
            }
        }
    }

    Ok(())
}

/// Get first string value from a KDL node
fn get_first_string(node: &KdlNode) -> Option<String> {
    if let Some(entry) = node.entries().first()
        && let Some(val) = entry.value().as_string()
    {
        return Some(val.to_string());
    }
    None
}

/// Extract packages from a node and add them to a target vector
///
/// Handles:
/// - String arguments: `packages "bat" "exa"`
/// - Children node names: `packages { bat exa }`
/// - Mixed: `packages "bat" { exa }`
fn extract_packages_to(node: &KdlNode, target: &mut Vec<PackageEntry>) {
    // Extract from string arguments of this node
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(PackageEntry {
                name: val.to_string(),
            });
        }
    }

    // Extract from children node names
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_name = child.name().value();
            let child_entries: Vec<_> = child
                .entries()
                .iter()
                .filter_map(|e| e.value().as_string())
                .collect();

            if child_entries.is_empty() {
                // No string arguments, just the node name
                target.push(PackageEntry {
                    name: child_name.to_string(),
                });
            } else {
                // Has string arguments - push node name AND all arguments
                // First, push the node name
                target.push(PackageEntry {
                    name: child_name.to_string(),
                });
                // Then, push all arguments as separate packages
                for arg in child_entries {
                    target.push(PackageEntry {
                        name: arg.to_string(),
                    });
                }
            }
        }
    }
}

fn extract_mixed_values(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

fn extract_mixed_values_return(node: &KdlNode) -> Vec<String> {
    let mut result = Vec::new();
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            result.push(val.to_string());
        }
    }

    if let Some(children) = node.children() {
        for child in children.nodes() {
            result.push(child.name().value().to_string());
            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    result.push(val.to_string());
                }
            }
        }
    }
    result
}

fn extract_strings(node: &KdlNode, target: &mut Vec<String>) {
    for entry in node.entries() {
        if let Some(val) = entry.value().as_string() {
            target.push(val.to_string());
        }
    }
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();
            target.push(name.to_string());

            for entry in child.entries() {
                if let Some(val) = entry.value().as_string() {
                    target.push(val.to_string());
                }
            }
        }
    }
}

/// Extract package aliases from KDL node
///
/// Supported syntax:
/// ```kdl
/// aliases-pkg pipewire pipewire-jack2
/// aliases-pkg {
///     python-poetry python-poetry-core
///     firefox firefox-beta
/// }
/// ```
fn extract_aliases(node: &KdlNode, target: &mut HashMap<String, String>) {
    // Case 1: Inline format: aliases-pkg name1 name2
    let entries: Vec<_> = node
        .entries()
        .iter()
        .filter_map(|e| e.value().as_string())
        .collect();

    if entries.len() == 2 {
        // aliases-pkg config_name actual_name
        target.insert(entries[0].to_string(), entries[1].to_string());
    }

    // Case 2: Children format
    if let Some(children) = node.children() {
        for child in children.nodes() {
            let child_entries: Vec<_> = child
                .entries()
                .iter()
                .filter_map(|e| e.value().as_string())
                .collect();

            if child_entries.len() == 2 {
                target.insert(child_entries[0].to_string(), child_entries[1].to_string());
            } else if child_entries.len() == 1 {
                // Support: node-name target-name
                let config_name = child.name().value();
                target.insert(config_name.to_string(), child_entries[0].to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests (unchanged for backward compatibility)

    #[test]
    fn test_parse_aliases_inline() {
        let kdl = r#"
            aliases-pkg pipewire pipewire-jack2
            aliases-pkg python-poetry python-poetry-core
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.aliases.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn test_parse_aliases_block() {
        let kdl = r#"
            aliases-pkg {
                pipewire pipewire-jack2
                python-poetry python-poetry-core
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
        assert_eq!(
            config.aliases.get("python-poetry"),
            Some(&"python-poetry-core".to_string())
        );
    }

    #[test]
    fn test_parse_mixed_config() {
        let kdl = r#"
            packages {
                neovim
                hyprland
            }

            aliases-pkg pipewire pipewire-jack2

            excludes bad-package
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.aliases.len(), 1);
        assert_eq!(config.excludes.len(), 1);
        assert_eq!(
            config.aliases.get("pipewire"),
            Some(&"pipewire-jack2".to_string())
        );
    }

    #[test]
    fn test_parse_empty_aliases() {
        let kdl = r#"
            packages neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_parse_aur_packages() {
        let kdl = r#"
            aur-packages {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
    }

    #[test]
    fn test_parse_default_packages() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                swww
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
        assert!(config.packages.iter().any(|p| p.name == "swww"));
    }

    #[test]
    fn test_parse_soar_packages() {
        let kdl = r#"
            soar-packages {
                bat
                exa
                ripgrep
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 3);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));
        assert!(config.soar_packages.iter().any(|p| p.name == "ripgrep"));
    }

    #[test]
    fn test_parse_flatpak_packages() {
        let kdl = r#"
            flatpak-packages {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn test_parse_cross_distro_config() {
        let kdl = r#"
            // Cross-distro configuration example

            // AUR packages (default, Arch-only)
            packages {
                hyprland
                waybar
                swww
            }

            // Soar packages (cross-distro static binaries)
            soar-packages {
                bat
                exa
                fd
                ripgrep
            }

            // Flatpak packages (cross-distro)
            flatpak-packages {
                com.spotify.Client
                org.telegram.desktop
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 3);
        assert_eq!(config.soar_packages.len(), 4);
        assert_eq!(config.flatpak_packages.len(), 2);
    }

    // New syntax tests

    #[test]
    fn test_parse_colon_syntax_soar() {
        let kdl = r#"
            packages:soar {
                bat
                exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn test_parse_colon_syntax_aur() {
        let kdl = r#"
            packages:aur {
                hyprland
                waybar
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
    }

    #[test]
    fn test_parse_colon_syntax_flatpak() {
        let kdl = r#"
            packages:flatpak {
                com.spotify.Client
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn test_parse_embedded_syntax() {
        let kdl = r#"
            packages {
                hyprland
                waybar
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                    org.mozilla.firefox
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 2);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "org.mozilla.firefox")
        );
    }

    #[test]
    fn test_parse_mixed_syntax_styles() {
        let kdl = r#"
            // Default packages (AUR)
            packages {
                hyprland
                waybar
            }

            // Colon syntax for Soar
            packages:soar {
                bat
            }

            // Colon syntax for Flatpak
            packages:flatpak {
                com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.soar_packages.len(), 1);
        assert_eq!(config.flatpak_packages.len(), 1);
    }

    #[test]
    fn test_parse_embedded_with_explicit_soar() {
        let kdl = r#"
            packages {
                soar {
                    bat
                    exa
                }
                flatpak {
                    com.spotify.Client
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    // NEW TESTS: Inline prefix syntax

    #[test]
    fn test_parse_inline_prefix_single() {
        let kdl = r#"
            packages {
                soar:bat
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
    }

    #[test]
    fn test_parse_inline_prefix_multiple() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar:bat
                soar:exa
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Default (hyprland) + aur:waybar
        assert_eq!(config.packages.len(), 2);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn test_parse_inline_prefix_with_nested_blocks() {
        let kdl = r#"
            packages {
                hyprland
                aur:waybar
                soar {
                    bat
                }
                flatpak:com.spotify.Client
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2); // hyprland + waybar
        assert_eq!(config.soar_packages.len(), 1); // bat
        assert_eq!(config.flatpak_packages.len(), 1); // com.spotify.Client
    }

    #[test]
    fn test_parse_inline_prefix_with_app_alias() {
        let kdl = r#"
            packages {
                app:bat
                app:exa
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));
    }

    #[test]
    fn test_parse_inline_prefix_string_arguments() {
        let kdl = r#"
            packages "soar:bat" "aur:hyprland" "flatpak:app.id"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.soar_packages.len(), 1);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));

        assert_eq!(config.packages.len(), 1);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));

        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(config.flatpak_packages.iter().any(|p| p.name == "app.id"));
    }

    #[test]
    fn test_parse_unknown_backend_with_inline_prefix() {
        let kdl = r#"
            packages {
                unknown:package
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        // Unknown backend should be treated as package name with default backend
        assert!(config.packages.iter().any(|p| p.name == "unknown:package"));
    }

    #[test]
    fn test_parse_complex_mixed_syntax() {
        let kdl = r#"
            packages {
                // Default packages (AUR)
                hyprland
                waybar

                // Inline prefix syntax
                soar:bat
                flatpak:com.spotify.Client

                // Nested blocks
                aur {
                    swww
                }

                // Mixed inline and nested
                soar {
                    exa
                }
                aur:rofi
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // AUR packages: hyprland, waybar, swww, rofi
        assert_eq!(config.packages.len(), 4);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));
        assert!(config.packages.iter().any(|p| p.name == "swww"));
        assert!(config.packages.iter().any(|p| p.name == "rofi"));

        // Soar packages: bat, exa
        assert_eq!(config.soar_packages.len(), 2);
        assert!(config.soar_packages.iter().any(|p| p.name == "bat"));
        assert!(config.soar_packages.iter().any(|p| p.name == "exa"));

        // Flatpak packages: com.spotify.Client
        assert_eq!(config.flatpak_packages.len(), 1);
        assert!(
            config
                .flatpak_packages
                .iter()
                .any(|p| p.name == "com.spotify.Client")
        );
    }

    #[test]
    fn test_backend_parser_registry() {
        let registry = BackendParserRegistry::new();

        // Test finding parsers by name
        assert!(registry.find_parser("aur").is_some());
        assert!(registry.find_parser("soar").is_some());
        assert!(registry.find_parser("flatpak").is_some());
        assert!(registry.find_parser("unknown").is_none());

        // Test aliases
        assert!(registry.find_parser("app").is_some()); // alias for soar
    }

    #[test]
    fn test_backward_compatibility() {
        // Ensure all old syntax still works
        let kdl = r#"
            packages {
                hyprland
                waybar
            }

            packages:soar {
                bat
            }

            packages:flatpak {
                com.spotify.Client
            }

            soar-packages {
                exa
            }

            flatpak-packages {
                org.mozilla.firefox
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.packages.len(), 2);
        assert_eq!(config.soar_packages.len(), 2); // bat + exa
        assert_eq!(config.flatpak_packages.len(), 2);
    }

    // NEW: Meta block tests

    #[test]
    fn test_parse_meta_block() {
        let kdl = r#"
            meta {
                description "My Hyprland Setup"
                author "nixval"
                version "1.0.0"
                url "https://github.com/nixval/dotfiles"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(
            config.meta.description,
            Some("My Hyprland Setup".to_string())
        );
        assert_eq!(config.meta.author, Some("nixval".to_string()));
        assert_eq!(config.meta.version, Some("1.0.0".to_string()));
        assert_eq!(
            config.meta.url,
            Some("https://github.com/nixval/dotfiles".to_string())
        );
    }

    #[test]
    fn test_parse_meta_with_tags() {
        let kdl = r#"
            meta {
                description "Workstation setup"
                tags "workstation" "hyprland" "development"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.meta.tags.len(), 3);
        assert!(config.meta.tags.contains(&"workstation".to_string()));
        assert!(config.meta.tags.contains(&"hyprland".to_string()));
        assert!(config.meta.tags.contains(&"development".to_string()));
    }

    // NEW: Conflicts tests

    #[test]
    fn test_parse_conflicts() {
        let kdl = r#"
            conflicts vim neovim
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 2);
        assert!(config.conflicts[0].packages.contains(&"vim".to_string()));
        assert!(config.conflicts[0].packages.contains(&"neovim".to_string()));
    }

    // NEW: Backend options tests

    #[test]
    fn test_parse_backend_options() {
        let kdl = r#"
            options:aur {
                noconfirm
                helper "paru"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.backend_options.contains_key("aur"));
        let aur_opts = &config.backend_options["aur"];
        assert_eq!(aur_opts.get("noconfirm"), Some(&"true".to_string()));
        assert_eq!(aur_opts.get("helper"), Some(&"paru".to_string()));
    }

    // NEW: Environment variables tests

    #[test]
    fn test_parse_env_vars() {
        let kdl = r#"
            env EDITOR="nvim" VISUAL="nvim"

            env:aur MAKEFLAGS="-j4"
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.env.contains_key("global"));
        assert!(config.env.contains_key("aur"));

        let global_env = &config.env["global"];
        assert!(global_env.contains(&"EDITOR=nvim".to_string()));
        assert!(global_env.contains(&"VISUAL=nvim".to_string()));

        let aur_env = &config.env["aur"];
        assert!(aur_env.contains(&"MAKEFLAGS=-j4".to_string()));
    }

    // NEW: Repositories tests

    #[test]
    fn test_parse_repositories() {
        let kdl = r#"
            repos:aur {
                "https://aur.archlinux.org"
            }

            repos:flatpak {
                "https://flathub.org/repo/flathub.flatpakrepo"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.repositories.contains_key("aur"));
        assert!(config.repositories.contains_key("flatpak"));

        assert!(config.repositories["aur"].contains(&"https://aur.archlinux.org".to_string()));
        assert!(
            config.repositories["flatpak"]
                .contains(&"https://flathub.org/repo/flathub.flatpakrepo".to_string())
        );
    }

    // NEW: Policy tests

    #[test]
    fn test_parse_policy() {
        let kdl = r#"
            policy {
                protected {
                    linux
                    systemd
                    base-devel
                }
                orphans "ask"
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert!(config.policy.protected.contains("linux"));
        assert!(config.policy.protected.contains("systemd"));
        assert!(config.policy.protected.contains("base-devel"));
        assert_eq!(config.policy.orphans, Some("ask".to_string()));
    }

    // NEW: Hooks tests

    #[test]
    fn test_parse_hooks() {
        let kdl = r#"
            hooks {
                post-sync {
                    run "notify-send 'Packages updated'"
                    sudo-needed "systemctl restart gdm"
                    script "~/.config/declarch/post-sync.sh"
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();
        assert_eq!(config.hooks.post_sync.len(), 3);

        assert_eq!(
            config.hooks.post_sync[0].command,
            "notify-send 'Packages updated'"
        );
        assert_eq!(config.hooks.post_sync[0].hook_type, HookType::Run);

        assert_eq!(config.hooks.post_sync[1].command, "systemctl restart gdm");
        assert_eq!(config.hooks.post_sync[1].hook_type, HookType::SudoNeeded);

        assert_eq!(
            config.hooks.post_sync[2].command,
            "~/.config/declarch/post-sync.sh"
        );
        assert_eq!(config.hooks.post_sync[2].hook_type, HookType::Script);
    }

    // NEW: Comprehensive integration test

    #[test]
    fn test_parse_full_config() {
        let kdl = r#"
            meta {
                description "Full workstation setup"
                author "nixval"
                version "2.0.0"
            }

            packages {
                hyprland
                neovim
                waybar
            }

            packages:soar {
                bat
                exa
            }

            conflicts {
                vim neovim
                pipewire pulseaudio
            }

            options:aur {
                noconfirm
            }

            env EDITOR="nvim"

            policy {
                protected {
                    linux
                    systemd
                }
                orphans "keep"
            }

            hooks {
                post-sync {
                    run "notify-send 'Sync complete'"
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check meta
        assert_eq!(
            config.meta.description,
            Some("Full workstation setup".to_string())
        );
        assert_eq!(config.meta.author, Some("nixval".to_string()));
        assert_eq!(config.meta.version, Some("2.0.0".to_string()));

        // Check packages
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.iter().any(|p| p.name == "hyprland"));
        assert!(config.packages.iter().any(|p| p.name == "neovim"));
        assert!(config.packages.iter().any(|p| p.name == "waybar"));

        // Check conflicts (1 conflict entry with 4 packages all mutually exclusive)
        assert_eq!(config.conflicts.len(), 1);
        assert_eq!(config.conflicts[0].packages.len(), 4);

        // Check options
        assert!(config.backend_options.contains_key("aur"));

        // Check env
        assert!(config.env.contains_key("global"));

        // Check policy
        assert!(config.policy.protected.contains("linux"));
        assert_eq!(config.policy.orphans, Some("keep".to_string()));

        // Check hooks
        assert_eq!(config.hooks.post_sync.len(), 1);
    }

    // NEW: Flat hooks syntax test

    #[test]
    fn test_parse_flat_hooks() {
        let kdl = r#"
            on-sync "notify-send 'Packages updated'"
            on-sync-sudo "systemctl restart gdm"
            on-pre-sync "echo 'Starting sync...'"
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Check pre-sync hooks
        assert_eq!(config.hooks.pre_sync.len(), 1);
        assert_eq!(config.hooks.pre_sync[0].command, "echo 'Starting sync...'");
        assert_eq!(config.hooks.pre_sync[0].hook_type, HookType::Run);

        // Check post-sync hooks
        assert_eq!(config.hooks.post_sync.len(), 2);
        assert_eq!(
            config.hooks.post_sync[0].command,
            "notify-send 'Packages updated'"
        );
        assert_eq!(config.hooks.post_sync[0].hook_type, HookType::Run);
        assert_eq!(config.hooks.post_sync[1].command, "systemctl restart gdm");
        assert_eq!(config.hooks.post_sync[1].hook_type, HookType::SudoNeeded);
    }

    // NEW: Mixed hooks (old nested + new flat)

    #[test]
    fn test_parse_mixed_hooks() {
        let kdl = r#"
            on-sync "notify-send 'Flat hook'"

            hooks {
                post-sync {
                    run "notify-send 'Nested hook'"
                }
            }
        "#;

        let config = parse_kdl_content(kdl).unwrap();

        // Should have both flat and nested hooks
        assert_eq!(config.hooks.post_sync.len(), 2);
        assert!(
            config
                .hooks
                .post_sync
                .iter()
                .any(|h| h.command == "notify-send 'Flat hook'")
        );
        assert!(
            config
                .hooks
                .post_sync
                .iter()
                .any(|h| h.command == "notify-send 'Nested hook'")
        );
    }
}
