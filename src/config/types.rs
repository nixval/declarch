use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(short, long, global = true, value_name = "DIR")]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Synchronize the system state
    Sync(SyncArgs),
    /// Manage configuration modules
    Module(ModuleArgs),
    Init,
}

#[derive(Parser, Debug)]
pub struct SyncArgs {
    /// Remove packages not present in configuration (orphans)
    #[arg(long)]
    pub prune: bool,
}

#[derive(Parser, Debug)]
pub struct ModuleArgs {
    #[command(subcommand)]
    pub command: ModuleCommand,
}

#[derive(Subcommand, Debug)]
pub enum ModuleCommand {
    /// List all available modules and their status
    List,
    /// Enable a module
    Enable { name: String },
    /// Disable a module
    Disable { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageRequest {
    pub name: String,
    pub version: Option<String>,
}

impl PackageRequest {
    pub fn from_kdl_string(pkg_str: &str) -> Self {
        if let Some((name, version)) = pkg_str.split_once('=') {
            PackageRequest {
                name: name.to_string(),
                version: Some(version.to_string()),
            }
        } else {
            PackageRequest {
                name: pkg_str.to_string(),
                version: None,
            }
        }
    }

    pub fn to_paru_string(&self) -> String {
        if let Some(version) = &self.version {
            format!("{}={}", self.name, version)
        } else {
            self.name.clone()
        }
    }
}
