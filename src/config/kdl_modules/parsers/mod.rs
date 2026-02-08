//! # Parsers
//!
//! This module provides parsers for different configuration formats.
//!
//! ## Architecture (v0.6+)
//!
//! In v0.6+, the parser is fully generic. There are no backend-specific parsers.
//! All packages are parsed using unified syntax:
//!
//! ```kdl
//! pkg {
//!   paru { hyprland waybar }
//!   npm { typescript eslint }
//! }
//! ```
//!
//! Or with colon syntax:
//!
//! ```kdl
//! pkg:paru { hyprland waybar }
//! ```
//!
//! The actual backend validation happens at runtime when the configuration
//! is loaded and the backend definitions are checked.

// All parsing is now handled by the generic parser in parser.rs
// No backend-specific parsers exist in v0.6+
