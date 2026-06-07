//! Embedding provider ports.

use async_trait::async_trait;

use crate::error::Result;
use crate::value_objects::Embedding;

/// AI Semantic Understanding Interface.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Create a single vector embedding for the given text.
    ///
    /// # Errors
    /// Returns an error if the embedding provider fails.
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_owned()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::embedding("No embedding returned"))
    }

    /// Create vector embeddings for a batch of strings.
    ///
    /// # Errors
    /// Returns an error if the embedding provider fails.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    /// Get the number of dimensions in the output vectors.
    fn dimensions(&self) -> usize;
    /// Get the name of this embedding provider.
    fn provider_name(&self) -> &str;

    /// Perform a basic health check on the embedding provider.
    ///
    /// # Errors
    /// Returns an error if the health check fails.
    async fn health_check(&self) -> Result<()> {
        self.embed("health check").await?;
        Ok(())
    }
}
