//! Runtime defaults and execution flow configuration.
//!
//! Provides boot-time discovery of execution provenance defaults and
//! execution flow mode enumeration for MCP server dispatch.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use uuid::Uuid;

use mcb_domain::ports::VcsProvider;

/// Valid execution flow modes for MCP tool dispatch.
///
/// Determines how the MCP server processes requests: direct stdio,
/// client-bridged to a running HTTP daemon, or server-managed hybrid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFlow {
    /// Direct stdio transport with no HTTP server.
    StdioOnly,
    /// Client bridges stdio calls to a running HTTP server.
    ClientHybrid,
    /// Server manages both HTTP and background stdio transport.
    ServerHybrid,
}

impl ExecutionFlow {
    /// Wire-format string for this execution flow.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StdioOnly => "stdio-only",
            Self::ClientHybrid => "client-hybrid",
            Self::ServerHybrid => "server-hybrid",
        }
    }
}

impl std::fmt::Display for ExecutionFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ExecutionFlow {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stdio-only" => Ok(Self::StdioOnly),
            "client-hybrid" => Ok(Self::ClientHybrid),
            "server-hybrid" => Ok(Self::ServerHybrid),
            other => Err(format!(
                "Invalid execution_flow '{other}'. Expected one of: {}, {}, {}",
                Self::StdioOnly.as_str(),
                Self::ClientHybrid.as_str(),
                Self::ServerHybrid.as_str(),
            )),
        }
    }
}

/// Boot-time execution provenance defaults used by context resolution.
#[derive(Debug, Clone)]
pub struct RuntimeDefaults {
    /// Workspace root discovered from the current working directory.
    pub workspace_root: Option<String>,
    /// Default repository path for tool execution.
    pub repo_path: Option<String>,
    /// Default repository identifier.
    pub repo_id: Option<String>,
    /// Default operator identifier.
    pub operator_id: Option<String>,
    /// Default machine identifier.
    pub machine_id: Option<String>,
    /// Default session identifier.
    pub session_id: Option<String>,
    /// Default agent program identifier.
    pub agent_program: Option<String>,
    /// Default model identifier.
    pub model_id: Option<String>,
    /// Default execution flow.
    pub execution_flow: Option<ExecutionFlow>,
}

impl RuntimeDefaults {
    /// Discover runtime defaults once at server boot.
    pub async fn discover(vcs: &dyn VcsProvider, execution_flow: Option<ExecutionFlow>) -> Self {
        let cwd = std::env::current_dir().ok();
        Self::discover_from_path(vcs, cwd.as_deref(), execution_flow).await
    }

    /// Discover runtime defaults from a given path.
    ///
    /// # Arguments
    /// * `vcs` - VCS provider for repository discovery
    /// * `cwd` - Current working directory (optional)
    /// * `execution_flow` - Execution flow mode (optional)
    ///
    /// # Returns
    /// `RuntimeDefaults` with discovered values
    pub async fn discover_from_path(
        vcs: &dyn VcsProvider,
        cwd: Option<&Path>,
        execution_flow: Option<ExecutionFlow>,
    ) -> Self {
        let workspace_root = match cwd {
            Some(path) => discover_workspace_root(vcs, path).await,
            None => None,
        };

        let repo_path = workspace_root.clone();
        let repo_id = if let Some(path) = workspace_root.as_deref() {
            vcs.open_repository(Path::new(path))
                .await
                .ok()
                .map(|repo| vcs.repository_id(&repo).into_string())
        } else {
            None
        };

        let machine_id = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .or_else(|| std::env::var("HOSTNAME").ok());

        Self {
            workspace_root,
            repo_path,
            repo_id,
            operator_id: std::env::var("USER").ok(),
            machine_id,
            session_id: Some(Uuid::new_v4().to_string()),
            agent_program: Some("mcb-stdio".to_owned()),
            model_id: Some("unknown".to_owned()),
            execution_flow,
        }
    }
}

/// Discover the workspace root by walking up the directory tree.
async fn discover_workspace_root(vcs: &dyn VcsProvider, cwd: &Path) -> Option<String> {
    let mut discovered_root: Option<PathBuf> = None;

    for candidate in cwd.ancestors() {
        if vcs.open_repository(candidate).await.is_ok() {
            discovered_root = Some(candidate.to_path_buf());
            continue;
        }

        if discovered_root.is_some() {
            break;
        }
    }

    discovered_root.map(|path| path.to_string_lossy().into_owned())
}
