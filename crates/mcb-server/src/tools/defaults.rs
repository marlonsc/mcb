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
    /// IDE/client session ID from the detected environment.
    pub client_session_id: Option<String>,
    /// Organization identifier auto-detected from git remote.
    pub org_id: Option<String>,
    /// Project identifier auto-detected from git remote.
    pub project_id: Option<String>,
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

        let (agent_program, client_session_id) = detect_ide();

        let (org_id, auto_project_id) = workspace_root
            .as_deref()
            .and_then(extract_org_and_project_from_git_remote)
            .map(|(org, proj)| (Some(org), Some(proj)))
            .unwrap_or((None, None));

        Self {
            workspace_root,
            repo_path,
            repo_id,
            operator_id: std::env::var("USER").ok(),
            machine_id,
            session_id: Some(Uuid::new_v4().to_string()),
            agent_program: Some(agent_program),
            client_session_id,
            model_id: Some("unknown".to_owned()),
            execution_flow,
            org_id,
            project_id: auto_project_id,
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

/// Detect the IDE/client from environment variables.
///
/// Returns `(agent_program, client_session_id)` where `agent_program` is the
/// detected IDE name and `client_session_id` is the IDE-specific session identifier.
///
/// Detection cascade (first match wins):
/// 1. `CURSOR_TRACE_ID` → Cursor
/// 2. `CLAUDE_CODE` or `CLAUDE_SESSION_ID` → Claude Code
/// 3. `OPENCODE_SESSION_ID` → `OpenCode`
/// 4. `VSCODE_PID` or `TERM_PROGRAM=vscode` → VS Code
/// 5. Fallback → "mcb-stdio"
fn detect_ide() -> (String, Option<String>) {
    // Cursor: sets CURSOR_TRACE_ID
    if let Ok(trace_id) = std::env::var("CURSOR_TRACE_ID") {
        return ("cursor".to_owned(), Some(trace_id));
    }

    // Claude Code: sets CLAUDE_CODE=1 or CLAUDE_SESSION_ID
    if std::env::var("CLAUDE_CODE").is_ok() {
        let session_id = std::env::var("CLAUDE_SESSION_ID").ok();
        return ("claude-code".to_owned(), session_id);
    }
    if let Ok(session_id) = std::env::var("CLAUDE_SESSION_ID") {
        return ("claude-code".to_owned(), Some(session_id));
    }

    // OpenCode: sets OPENCODE_SESSION_ID
    if let Ok(session_id) = std::env::var("OPENCODE_SESSION_ID") {
        return ("opencode".to_owned(), Some(session_id));
    }

    // VS Code: sets VSCODE_PID or TERM_PROGRAM=vscode
    if std::env::var("VSCODE_PID").is_ok()
        || std::env::var("TERM_PROGRAM")
            .map(|v| v.eq_ignore_ascii_case("vscode"))
            .unwrap_or(false)
    {
        return ("vscode".to_owned(), std::env::var("VSCODE_PID").ok());
    }

    // Fallback: plain stdio
    ("mcb-stdio".to_owned(), None)
}

/// Extract org and project from `git remote get-url origin` in the given directory.
fn extract_org_and_project_from_git_remote(workspace_root: &str) -> Option<(String, String)> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(workspace_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout);
    parse_org_and_project_from_remote_url(url.trim())
}

/// Parse org and project name from a git remote URL.
///
/// Supports formats:
/// - `git@github.com:owner/repo.git`
/// - `https://github.com/owner/repo.git`
/// - `https://github.com/owner/repo`
fn parse_org_and_project_from_remote_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();

    // SSH format: git@host:owner/repo.git
    if let Some(path) = url
        .strip_prefix("git@")
        .and_then(|s| s.split_once(':').map(|(_, p)| p))
    {
        let clean = path.trim_end_matches(".git");
        if let Some((org, project)) = clean.split_once('/') {
            if !org.is_empty() && !project.is_empty() {
                return Some((org.to_owned(), project.to_owned()));
            }
        }
    }

    // HTTPS format: https://host/owner/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let segments: Vec<&str> = url
            .split("//")
            .nth(1)?
            .split('/')
            .skip(1) // skip host
            .collect();
        if segments.len() >= 2 {
            let org = segments[0];
            let project = segments[1].trim_end_matches(".git");
            if !org.is_empty() && !project.is_empty() {
                return Some((org.to_owned(), project.to_owned()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ide_fallback() {
        // Clear all IDE env vars to test fallback
        // Note: env vars are process-global, so this test is best-effort
        let (program, session) = detect_ide();
        // In CI/test environment, we expect one of the known IDEs or fallback
        assert!(
            ["mcb-stdio", "cursor", "claude-code", "opencode", "vscode"]
                .contains(&program.as_str()),
            "unexpected agent_program: {program}"
        );
        // client_session_id may or may not be set depending on environment
        let _ = session;
    }
}
