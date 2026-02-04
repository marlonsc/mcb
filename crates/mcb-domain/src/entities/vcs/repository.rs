use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId(pub String);

impl RepositoryId {
    #[must_use]
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RepositoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RepositoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsRepository {
    pub id: RepositoryId,
    pub path: PathBuf,
    pub default_branch: String,
    pub branches: Vec<String>,
    pub remote_url: Option<String>,
}
