//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! IDE / Agent program identifier constants — Single Source of Truth
//!
//! These constants identify the client IDE or agent program that
//! initiated the MCP session. Used across `defaults.rs`, `mcp_server.rs`,
//! `http_client.rs`, and tests.

define_str_consts! {
    /// Cursor IDE identifier.
    IDE_CURSOR = "cursor";
    /// Claude Code IDE identifier.
    IDE_CLAUDE_CODE = "claude-code";
    /// OpenCode IDE identifier.
    IDE_OPENCODE = "opencode";
    /// VS Code IDE identifier.
    IDE_VSCODE = "vscode";
    /// MCB stdio fallback — no IDE detected.
    IDE_MCB_STDIO = "mcb-stdio";
    /// MCB HTTP client bridge agent.
    IDE_MCB_CLIENT = "mcb-client";
}

/// All known IDE / agent program identifiers.
///
/// Used for validation and test assertions.
pub const KNOWN_IDE_PROGRAMS: &[&str] = &[
    IDE_MCB_STDIO,
    IDE_CURSOR,
    IDE_CLAUDE_CODE,
    IDE_OPENCODE,
    IDE_VSCODE,
];
