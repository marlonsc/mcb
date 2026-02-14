use serde::{Deserialize, Serialize};

/// Workspace-level project settings loaded from `.mcb/project.toml`
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProjectSettings {
    /// Project name
    pub name: Option<String>,
    /// Project description
    pub description: Option<String>,
    /// Provider overrides
    pub providers: Option<ProjectProvidersSettings>,
    /// Files to ignore
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Provider overrides specified at the project level
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProjectProvidersSettings {
    /// Embedding provider override
    pub embedding: Option<ProjectEmbeddingConfig>,
    /// Vector store provider override
    pub vector_store: Option<ProjectVectorStoreConfig>,
}

/// Project embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProjectEmbeddingConfig {
    /// Embedding provider name (e.g. "openai", "ollama")
    pub provider: Option<String>,
    /// Embedding model name
    pub model: Option<String>,
}

/// Project vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProjectVectorStoreConfig {
    /// Vector store provider name (e.g. "edgevec", "qdrant")
    pub provider: Option<String>,
    /// Collection name override
    pub collection: Option<String>,
}
