use std::process::Command;
use std::sync::OnceLock;

use mcb_domain::utils::vcs_context::VcsContext;
use mcb_domain::value_objects::project_context::{ProjectContext, parse_owner_repo};

static VCS_CONTEXT: OnceLock<VcsContext> = OnceLock::new();
static PROJECT_CONTEXT: OnceLock<Option<ProjectContext>> = OnceLock::new();

/// Capture VCS context (branch, commit, repo) from the git environment. Cached after first call.
#[must_use]
pub fn capture_vcs_context() -> VcsContext {
    VCS_CONTEXT
        .get_or_init(|| {
            let (branch, commit) = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD", "HEAD"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        let output = String::from_utf8_lossy(&o.stdout);
                        let mut lines = output.lines();
                        let branch = lines.next().map(|s| s.trim().to_string());
                        let commit = lines.next().map(|s| s.trim().to_string());
                        Some((branch, commit))
                    } else {
                        None
                    }
                })
                .unwrap_or((None, None));

            let repo_id = Command::new("git")
                .args(["config", "--get", "remote.origin.url"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            VcsContext::new(branch, commit, repo_id)
        })
        .clone()
}

/// Resolve project identity from the current git repository. Cached after first call.
///
/// Returns `None` when the project cannot be detected from git context
/// (no remote origin, no toplevel directory). Callers must handle the
/// absence explicitly — falling back to a hardcoded identifier is a
/// security violation (project isolation depends on accurate identity).
#[must_use]
pub fn resolve_project_context() -> Option<ProjectContext> {
    PROJECT_CONTEXT.get_or_init(detect_project_context).clone()
}

fn detect_project_context() -> Option<ProjectContext> {
    let superproject_id = detect_superproject();
    let is_submodule = superproject_id.is_some();

    if let Some((project_id, project_name)) = project_context_from_git_remote() {
        return Some(ProjectContext {
            project_id,
            project_name,
            is_submodule,
            superproject_id,
        });
    }

    if let Some((project_id, project_name)) = project_context_from_git_toplevel() {
        return Some(ProjectContext {
            project_id,
            project_name,
            is_submodule,
            superproject_id,
        });
    }

    None
}

fn project_context_from_git_remote() -> Option<(String, String)> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return None;
    }

    let owner_repo = parse_owner_repo(&url)?;
    let name = owner_repo
        .rsplit('/')
        .next()
        .unwrap_or(&owner_repo)
        .to_string();
    Some((owner_repo, name))
}

fn project_context_from_git_toplevel() -> Option<(String, String)> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let name = std::path::Path::new(&toplevel)
        .file_name()
        .and_then(|n| n.to_str())?
        .to_string();
    Some((name.clone(), name))
}

fn detect_superproject() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-superproject-working-tree"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let superproject_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if superproject_path.is_empty() {
        return None;
    }

    let output = Command::new("git")
        .args([
            "-C",
            &superproject_path,
            "config",
            "--get",
            "remote.origin.url",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_owner_repo(&url)
}
