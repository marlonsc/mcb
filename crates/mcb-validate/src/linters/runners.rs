//! Linter Runners Module
//!
//! Concrete implementations for running specific linters.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use super::parsers::run_linter_command;
use super::types::{LintViolation, LinterType};
use crate::Result;

/// Execute Ruff linter on files
pub struct RuffLinter;

impl RuffLinter {
    /// Check multiple files using Ruff
    pub async fn check_files(files: &[&Path]) -> Result<Vec<LintViolation>> {
        let linter = LinterType::Ruff;
        let output = run_linter_command(linter, files).await?;
        Ok(linter.parse_output(&output))
    }

    /// Check a single file using Ruff
    pub async fn check_file(file: &Path) -> Result<Vec<LintViolation>> {
        Self::check_files(&[file]).await
    }
}

/// Execute Clippy linter on Rust project
pub struct ClippyLinter;

impl ClippyLinter {
    /// Check project with default Clippy lints
    pub async fn check_project(project_root: &Path) -> Result<Vec<LintViolation>> {
        Self::check_project_with_lints(project_root, &[]).await
    }

    /// Check project with specific lint codes enabled as warnings
    ///
    /// This is used by `YamlRuleExecutor` to enable specific lints from `lint_select`.
    /// For example, `clippy::unwrap_used` is "allow" by default and needs `-W` to enable.
    pub async fn check_project_with_lints(
        project_root: &Path,
        lint_codes: &[String],
    ) -> Result<Vec<LintViolation>> {
        let mut args = vec![
            "clippy".to_string(),
            "--message-format=json".to_string(),
            "--".to_string(),
        ];

        // Add each lint code as a warning flag
        for code in lint_codes {
            if code.starts_with("clippy::") {
                args.push("-W".to_string());
                args.push(code.clone());
            }
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(LinterType::Clippy.parse_output(&stdout))
    }
}
