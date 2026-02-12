//! DB-first repository resolution with auto-registration.
use std::sync::Arc;

use mcb_domain::entities::repository::{Repository, VcsType};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::VcsEntityRepository;
use mcb_domain::value_objects::project_context::{ProjectContext, normalize_repo_url};

/// DB-first project resolution: checks registered repositories before falling back to git.
pub struct RepositoryResolver {
    vcs_repo: Arc<dyn VcsEntityRepository>,
    project_ctx: ProjectContext,
}

impl RepositoryResolver {
    /// Create a resolver that detects the current git repository context.
    pub fn new(vcs_repo: Arc<dyn VcsEntityRepository>) -> Self {
        Self {
            vcs_repo,
            project_ctx: ProjectContext::resolve(),
        }
    }

    /// Create a resolver with a custom project context (for testing or custom setups).
    pub fn with_context(
        vcs_repo: Arc<dyn VcsEntityRepository>,
        project_ctx: ProjectContext,
    ) -> Self {
        Self {
            vcs_repo,
            project_ctx,
        }
    }

    /// Resolve `project_id` from DB or fall back to git-derived value.
    pub async fn resolve_project_id(&self, org_id: &str) -> String {
        match self.try_resolve(org_id).await {
            Ok(Some(id)) => id,
            _ => self.project_ctx.project_id.clone(),
        }
    }

    /// Resolve `project_id` and auto-register the repository if not yet known.
    pub async fn resolve_and_register(&self, org_id: &str) -> Result<String> {
        if let Some(id) = self.try_resolve(org_id).await? {
            return Ok(id);
        }

        let project_id = &self.project_ctx.project_id;

        if !Self::is_registerable(project_id) {
            return Ok(project_id.clone());
        }

        if self.project_ctx.is_submodule {
            return self.resolve_submodule(org_id).await;
        }

        self.register_repository(org_id, project_id, project_id)
            .await?;
        Ok(project_id.clone())
    }

    async fn try_resolve(&self, org_id: &str) -> Result<Option<String>> {
        let normalized_url = &self.project_ctx.project_id;
        if !Self::is_registerable(normalized_url) {
            return Ok(None);
        }

        let repo = self
            .vcs_repo
            .find_repository_by_url(org_id, normalized_url)
            .await?;
        Ok(repo.map(|r| r.project_id))
    }

    async fn resolve_submodule(&self, org_id: &str) -> Result<String> {
        let parent_url = self
            .project_ctx
            .superproject_id
            .as_deref()
            .unwrap_or("default");

        if !Self::is_registerable(parent_url) {
            return Ok(self.project_ctx.project_id.clone());
        }

        let parent_project_id = match self
            .vcs_repo
            .find_repository_by_url(org_id, parent_url)
            .await?
        {
            Some(parent_repo) => parent_repo.project_id,
            None => {
                self.register_repository(org_id, parent_url, parent_url)
                    .await?;
                parent_url.to_string()
            }
        };

        self.register_repository(org_id, &parent_project_id, &self.project_ctx.project_id)
            .await?;
        Ok(parent_project_id)
    }

    async fn register_repository(&self, org_id: &str, project_id: &str, url: &str) -> Result<()> {
        self.vcs_repo.ensure_org_and_project(project_id).await?;

        let name = url.rsplit('/').next().unwrap_or(url).to_string();
        let local_path = Self::git_toplevel().unwrap_or_default();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let repo = Repository {
            id: uuid::Uuid::new_v4().to_string(),
            org_id: org_id.to_string(),
            project_id: project_id.to_string(),
            name,
            url: normalize_repo_url(url),
            local_path,
            vcs_type: VcsType::Git,
            created_at: now,
            updated_at: now,
        };

        // INSERT OR IGNORE semantics: if another request already registered
        // this exact (org_id, project_id, name) combo, the unique constraint
        // rejects the duplicate harmlessly.
        let _ = self.vcs_repo.create_repository(&repo).await;
        Ok(())
    }

    fn is_registerable(project_id: &str) -> bool {
        project_id.contains('/') && project_id != "default"
    }

    fn git_toplevel() -> Option<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() { None } else { Some(path) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::constants::keys::DEFAULT_ORG_ID;
    use mcb_domain::entities::repository::Branch;
    use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
    use mcb_domain::error::Error;
    use std::sync::Mutex;

    struct MockVcsRepo {
        repos: Mutex<Vec<Repository>>,
    }

    impl MockVcsRepo {
        fn new() -> Self {
            Self {
                repos: Mutex::new(Vec::new()),
            }
        }

        fn with_repos(repos: Vec<Repository>) -> Self {
            Self {
                repos: Mutex::new(repos),
            }
        }

        fn repo_count(&self) -> usize {
            self.repos.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl VcsEntityRepository for MockVcsRepo {
        async fn create_repository(&self, repo: &Repository) -> Result<()> {
            self.repos.lock().unwrap().push(repo.clone());
            Ok(())
        }
        async fn get_repository(&self, _org_id: &str, _id: &str) -> Result<Repository> {
            Err(Error::not_found("not found"))
        }
        async fn find_repository_by_url(
            &self,
            org_id: &str,
            url: &str,
        ) -> Result<Option<Repository>> {
            Ok(self
                .repos
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.org_id == org_id && r.url == url)
                .cloned())
        }
        async fn list_repositories(
            &self,
            _org_id: &str,
            _project_id: &str,
        ) -> Result<Vec<Repository>> {
            Ok(vec![])
        }
        async fn update_repository(&self, _repo: &Repository) -> Result<()> {
            Ok(())
        }
        async fn delete_repository(&self, _org_id: &str, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn ensure_org_and_project(&self, _project_id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_branch(&self, _branch: &Branch) -> Result<()> {
            Ok(())
        }
        async fn get_branch(&self, _id: &str) -> Result<Branch> {
            Err(Error::not_found("not found"))
        }
        async fn list_branches(&self, _repository_id: &str) -> Result<Vec<Branch>> {
            Ok(vec![])
        }
        async fn update_branch(&self, _branch: &Branch) -> Result<()> {
            Ok(())
        }
        async fn delete_branch(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_worktree(&self, _wt: &Worktree) -> Result<()> {
            Ok(())
        }
        async fn get_worktree(&self, _id: &str) -> Result<Worktree> {
            Err(Error::not_found("not found"))
        }
        async fn list_worktrees(&self, _repository_id: &str) -> Result<Vec<Worktree>> {
            Ok(vec![])
        }
        async fn update_worktree(&self, _wt: &Worktree) -> Result<()> {
            Ok(())
        }
        async fn delete_worktree(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_assignment(&self, _asgn: &AgentWorktreeAssignment) -> Result<()> {
            Ok(())
        }
        async fn get_assignment(&self, _id: &str) -> Result<AgentWorktreeAssignment> {
            Err(Error::not_found("not found"))
        }
        async fn list_assignments_by_worktree(
            &self,
            _worktree_id: &str,
        ) -> Result<Vec<AgentWorktreeAssignment>> {
            Ok(vec![])
        }
        async fn release_assignment(&self, _id: &str, _released_at: i64) -> Result<()> {
            Ok(())
        }
    }

    fn make_repo(org_id: &str, project_id: &str, url: &str) -> Repository {
        Repository {
            id: uuid::Uuid::new_v4().to_string(),
            org_id: org_id.to_string(),
            project_id: project_id.to_string(),
            name: url.rsplit('/').next().unwrap_or(url).to_string(),
            url: url.to_string(),
            local_path: "/tmp/test".to_string(),
            vcs_type: VcsType::Git,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[tokio::test]
    async fn db_hit_returns_project_id() {
        let mock: Arc<dyn VcsEntityRepository> =
            Arc::new(MockVcsRepo::with_repos(vec![make_repo(
                DEFAULT_ORG_ID,
                "custom-project",
                "marlonsc/mcb",
            )]));
        let ctx = ProjectContext::new("marlonsc/mcb", "mcb");
        let resolver = RepositoryResolver::with_context(mock, ctx);

        let result = resolver.resolve_project_id(DEFAULT_ORG_ID).await;
        assert_eq!(result, "custom-project");
    }

    #[tokio::test]
    async fn db_miss_with_valid_url_auto_registers() {
        let mock = Arc::new(MockVcsRepo::new());
        let ctx = ProjectContext::new("marlonsc/mcb", "mcb");
        let resolver = RepositoryResolver::with_context(
            Arc::clone(&mock) as Arc<dyn VcsEntityRepository>,
            ctx,
        );

        let result = resolver.resolve_and_register(DEFAULT_ORG_ID).await.unwrap();
        assert_eq!(result, "marlonsc/mcb");
        assert_eq!(mock.repo_count(), 1);
    }

    #[tokio::test]
    async fn db_miss_submodule_with_parent_in_db() {
        let mock = Arc::new(MockVcsRepo::with_repos(vec![make_repo(
            DEFAULT_ORG_ID,
            "parent-project",
            "org/parent",
        )]));
        let mut ctx = ProjectContext::new("org/child", "child");
        ctx.is_submodule = true;
        ctx.superproject_id = Some("org/parent".to_string());
        let resolver = RepositoryResolver::with_context(
            Arc::clone(&mock) as Arc<dyn VcsEntityRepository>,
            ctx,
        );

        let result = resolver.resolve_and_register(DEFAULT_ORG_ID).await.unwrap();
        assert_eq!(result, "parent-project");
        assert_eq!(mock.repo_count(), 2);
    }

    #[tokio::test]
    async fn db_miss_submodule_parent_not_in_db() {
        let mock = Arc::new(MockVcsRepo::new());
        let mut ctx = ProjectContext::new("org/child", "child");
        ctx.is_submodule = true;
        ctx.superproject_id = Some("org/parent".to_string());
        let resolver = RepositoryResolver::with_context(
            Arc::clone(&mock) as Arc<dyn VcsEntityRepository>,
            ctx,
        );

        let result = resolver.resolve_and_register(DEFAULT_ORG_ID).await.unwrap();
        assert_eq!(result, "org/parent");
        assert_eq!(mock.repo_count(), 2);
    }

    #[tokio::test]
    async fn default_identifier_no_auto_registration() {
        let mock = Arc::new(MockVcsRepo::new());
        let ctx = ProjectContext::new("default", "default");
        let resolver = RepositoryResolver::with_context(
            Arc::clone(&mock) as Arc<dyn VcsEntityRepository>,
            ctx,
        );

        let result = resolver.resolve_and_register(DEFAULT_ORG_ID).await.unwrap();
        assert_eq!(result, "default");
        assert_eq!(mock.repo_count(), 0);
    }

    #[tokio::test]
    async fn directory_name_identifier_no_auto_registration() {
        let mock = Arc::new(MockVcsRepo::new());
        let ctx = ProjectContext::new("mcb", "mcb");
        let resolver = RepositoryResolver::with_context(
            Arc::clone(&mock) as Arc<dyn VcsEntityRepository>,
            ctx,
        );

        let result = resolver.resolve_and_register(DEFAULT_ORG_ID).await.unwrap();
        assert_eq!(result, "mcb");
        assert_eq!(mock.repo_count(), 0);
    }
}
