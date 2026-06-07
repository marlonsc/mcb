//! Observation storage and management operations.
//!
//! Handles storing observations in both relational and vector stores,
//! with deduplication via content hashing.

use std::collections::HashMap;

use mcb_domain::constants::keys::{
    METADATA_KEY_CONTENT, METADATA_KEY_FILE_PATH, METADATA_KEY_SESSION_ID, METADATA_KEY_START_LINE,
    METADATA_KEY_TAGS, METADATA_KEY_TYPE,
};
use mcb_domain::entities::memory::{Observation, ObservationMetadata, ObservationType};
use mcb_domain::error::Result;
use mcb_domain::utils::id;
use mcb_domain::utils::id::compute_content_hash;
use mcb_domain::utils::time as domain_time;
use mcb_domain::value_objects::CollectionId;

use crate::constants::use_cases::MEMORY_COLLECTION_NAME;

use super::MemoryServiceImpl;

impl MemoryServiceImpl {
    /// Store an observation in both relational and vector stores.
    ///
    /// Deduplicates based on content hash. Returns the observation ID and a boolean
    /// indicating whether the input was deduplicated (`true` means duplicate content).
    pub(crate) async fn store_observation_impl(
        &self,
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<(String, bool)> {
        if project_id.trim().is_empty() {
            return Err(mcb_domain::error::Error::invalid_argument(
                "Project ID cannot be empty for memory storage",
            ));
        }

        let content_hash = compute_content_hash(&content);

        if let Some(existing) = self.repository.find_by_hash(&content_hash).await? {
            return Ok((existing.id, true));
        }

        let embedding = self.embedding_provider.embed(&content).await?;

        let mut vector_metadata = HashMap::new();
        vector_metadata.insert(
            METADATA_KEY_CONTENT.to_owned(),
            serde_json::Value::String(content.clone()),
        );
        vector_metadata.insert(
            METADATA_KEY_TYPE.to_owned(),
            serde_json::Value::String(r#type.as_str().to_owned()),
        );
        vector_metadata.insert(METADATA_KEY_TAGS.to_owned(), serde_json::json!(tags));
        vector_metadata.insert(
            "project_id".to_owned(),
            serde_json::Value::String(project_id.clone()),
        );

        if let Some(session_id) = &metadata.session_id {
            vector_metadata.insert(
                METADATA_KEY_SESSION_ID.to_owned(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        // Vector stores (Milvus) require file_path and start_line columns
        // for all vectors. For observations, use the origin file_path if available,
        // otherwise default to "memory" sentinel so the insert doesn't fail.
        vector_metadata.insert(
            METADATA_KEY_FILE_PATH.to_owned(),
            serde_json::Value::String(
                metadata
                    .file_path
                    .clone()
                    .unwrap_or_else(|| "memory".to_owned()),
            ),
        );
        vector_metadata.insert(
            METADATA_KEY_START_LINE.to_owned(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );

        let collection_id =
            CollectionId::from_uuid(id::deterministic("collection", MEMORY_COLLECTION_NAME));
        let ids = self
            .vector_store
            .insert_vectors(&collection_id, &[embedding], vec![vector_metadata])
            .await?;

        let embedding_id = ids.first().cloned();

        let observation = Observation {
            id: id::generate().to_string(),
            project_id,
            content,
            content_hash,
            tags,
            r#type,
            metadata,
            created_at: domain_time::epoch_secs_i64()?,
            embedding_id,
        };

        if let Err(err) = self.repository.store_observation(&observation).await {
            let _ = self.vector_store.delete_vectors(&collection_id, &ids).await;
            return Err(err);
        }

        Ok((observation.id, false))
    }
}
