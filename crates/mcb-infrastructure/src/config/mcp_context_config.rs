//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// MCP Context configuration from .mcp-context.toml files
///
/// Enables per-repository configuration for git-aware indexing:
/// - branches: which branches to index (default: main, HEAD, current)
/// - depth: commit history depth (default: 50)
/// - `ignore_patterns`: patterns to exclude (e.g., "*.log", "`node_modules`/")
/// - `include_submodules`: recursive indexing (default: true)
use serde::{Deserialize, Serialize};
use thiserror::Error;

const EMBEDDED_APP_DEFAULTS: &str = include_str!("../../../../config/default.toml");

/// Configuration errors that can occur during MCP context setup.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The configuration file was not found at the expected path.
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    /// Failed to read the configuration file from disk.
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    /// The configuration file contains invalid TOML syntax.
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// Git configuration for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Branches to index (e.g., ["main", "develop"])
    pub branches: Vec<String>,

    /// Number of commits to index per branch
    #[serde(default = "default_depth")]
    pub depth: usize,

    /// Patterns to ignore (e.g., ["*.log", "`node_modules`/"])
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Include submodules in indexing
    #[serde(default = "default_include_submodules")]
    pub include_submodules: bool,
}

fn default_depth() -> usize {
    50
}

fn default_include_submodules() -> bool {
    true
}

/// Default Git configuration: main/HEAD branches, depth 50, submodules included.
impl Default for GitConfig {
    fn default() -> Self {
        Self {
            branches: vec!["main".to_owned(), "HEAD".to_owned(), "current".to_owned()],
            depth: default_depth(),
            ignore_patterns: Vec::new(),
            include_submodules: default_include_submodules(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct EmbeddedDefaultsToml {
    mcp_context: EmbeddedMcpContext,
}

#[derive(Debug, Deserialize)]
struct EmbeddedMcpContext {
    git: GitConfig,
}

fn embedded_mcp_context_defaults() -> Result<McpContextConfig, ConfigError> {
    let parsed: EmbeddedDefaultsToml = toml::from_str(EMBEDDED_APP_DEFAULTS)?;
    Ok(McpContextConfig {
        git: parsed.mcp_context.git,
        custom: HashMap::new(),
    })
}

/// Root MCP Context configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpContextConfig {
    /// Git-specific configuration
    #[serde(default)]
    pub git: GitConfig,

    /// Additional custom settings
    #[serde(flatten)]
    pub custom: HashMap<String, toml::Value>,
}

impl McpContextConfig {
    /// Load configuration from .mcp-context.toml file in given directory
    /// Returns default config if file not found (non-fatal)
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let config_path = path.join(".mcp-context.toml");

        if !config_path.exists() {
            return embedded_mcp_context_defaults();
        }

        let content = fs::read_to_string(&config_path)?;
        let config: McpContextConfig = toml::from_str(&content)?;

        Ok(config)
    }

    /// Load configuration, returning default if file not found
    #[must_use]
    pub fn load_from_path_or_default(path: &Path) -> Self {
        Self::load_from_path(path).unwrap_or_default()
    }
}
