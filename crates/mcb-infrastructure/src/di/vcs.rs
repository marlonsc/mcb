use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use mcb_domain::entities::vcs::{RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VcsProvider;
use mcb_domain::registry::vcs::{VcsProviderConfig, list_vcs_providers, resolve_vcs_provider};

#[allow(missing_docs)]
pub struct DynamicVcsProvider {
    providers: Vec<Arc<dyn VcsProvider>>,
}

impl DynamicVcsProvider {
    fn from_registry() -> Result<Self> {
        let mut providers = Vec::new();
        for (name, _) in list_vcs_providers() {
            providers.push(resolve_vcs_provider(&VcsProviderConfig::new(name))?);
        }

        if providers.is_empty() {
            return Err(Error::configuration(
                "VCS: no providers registered in linkme registry",
            ));
        }

        Ok(Self { providers })
    }

    async fn provider_and_repo_for_path(
        &self,
        path: &Path,
    ) -> Result<(Arc<dyn VcsProvider>, VcsRepository)> {
        let mut last_error: Option<Error> = None;

        for provider in &self.providers {
            match provider.open_repository(path).await {
                Ok(repo) => return Ok((Arc::clone(provider), repo)),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::vcs(format!(
                "No registered VCS provider can open path: {}",
                path.display()
            ))
        }))
    }
}

#[async_trait]
impl VcsProvider for DynamicVcsProvider {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        let (_, repo) = self.provider_and_repo_for_path(path).await?;
        Ok(repo)
    }

    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId {
        self.providers.first().map_or_else(
            || repo.id().clone(),
            |provider| provider.repository_id(repo),
        )
    }

    async fn list_branches(&self, repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
        let (provider, concrete_repo) = self.provider_and_repo_for_path(repo.path()).await?;
        provider.list_branches(&concrete_repo).await
    }

    async fn commit_history(
        &self,
        repo: &VcsRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        let (provider, concrete_repo) = self.provider_and_repo_for_path(repo.path()).await?;
        provider.commit_history(&concrete_repo, branch, limit).await
    }

    async fn list_files(&self, repo: &VcsRepository, branch: &str) -> Result<Vec<PathBuf>> {
        let (provider, concrete_repo) = self.provider_and_repo_for_path(repo.path()).await?;
        provider.list_files(&concrete_repo, branch).await
    }

    async fn read_file(&self, repo: &VcsRepository, branch: &str, path: &Path) -> Result<String> {
        let (provider, concrete_repo) = self.provider_and_repo_for_path(repo.path()).await?;
        provider.read_file(&concrete_repo, branch, path).await
    }

    fn vcs_name(&self) -> &str {
        "dynamic"
    }

    async fn diff_refs(
        &self,
        repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff> {
        let (provider, concrete_repo) = self.provider_and_repo_for_path(repo.path()).await?;
        provider.diff_refs(&concrete_repo, base_ref, head_ref).await
    }

    async fn list_repositories(&self, root: &Path) -> Result<Vec<VcsRepository>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();

        for provider in &self.providers {
            for repo in provider.list_repositories(root).await? {
                let key = repo.path().to_path_buf();
                if seen.insert(key) {
                    out.push(repo);
                }
            }
        }

        Ok(out)
    }
}

#[allow(missing_docs)]
pub fn default_vcs_provider() -> Result<Arc<dyn VcsProvider>> {
    Ok(Arc::new(DynamicVcsProvider::from_registry()?))
}
