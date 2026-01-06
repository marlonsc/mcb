//! Vector store provider implementations

use crate::core::{error::Result, types::Embedding};
use crate::providers::VectorStoreProvider;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory vector store provider for MVP
pub struct InMemoryVectorStoreProvider {
    collections: Mutex<HashMap<String, Vec<Embedding>>>,
}

impl InMemoryVectorStoreProvider {
    pub fn new() -> Self {
        Self {
            collections: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryVectorStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStoreProvider for InMemoryVectorStoreProvider {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        let coll = collections.entry(collection.to_string()).or_default();
        coll.extend_from_slice(embeddings);
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<(f32, Embedding)>> {
        let collections = self.collections.lock().unwrap();
        if let Some(coll) = collections.get(collection) {
            let mut results: Vec<_> = coll
                .iter()
                .map(|emb| {
                    let similarity = cosine_similarity(query, &emb.vector);
                    (similarity, emb.clone())
                })
                .collect();

            results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            results.truncate(limit);
            Ok(results)
        } else {
            Ok(vec![])
        }
    }

    async fn clear(&self, collection: &str) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        collections.remove(collection);
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "in-memory"
    }
}

/// Cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
