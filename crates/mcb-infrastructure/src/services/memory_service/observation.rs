//! Observation storage and management operations.
//!
//! Handles storing observations in both relational and vector stores,
//! with deduplication via content hashing.

use std::collections::HashMap;

use mcb_domain::entities::memory::{Observation, ObservationMetadata, ObservationType};
use mcb_domain::error::Result;
use mcb_domain::value_objects::CollectionId;
use mcb_utils::constants::keys::{
    METADATA_KEY_CONTENT, METADATA_KEY_FILE_PATH, METADATA_KEY_SESSION_ID, METADATA_KEY_START_LINE,
    METADATA_KEY_TAGS, METADATA_KEY_TYPE,
};
use mcb_utils::utils::id;
use mcb_utils::utils::id::compute_content_hash;
use mcb_utils::utils::time as domain_time;

use mcb_utils::constants::use_cases::MEMORY_COLLECTION_NAME;

use super::MemoryServiceImpl;

/// Owned inputs describing an observation to be stored.
pub(crate) struct ObservationInput {
    pub project_id: String,
    pub content: String,
    pub r#type: ObservationType,
    pub tags: Vec<String>,
    pub metadata: ObservationMetadata,
}

/// Result of persisting the observation embedding into the vector store.
///
/// Carries the data needed to finalize the relational record and to roll the
/// vectors back if the relational write fails.
struct VectorWriteOutcome<'a> {
    content_hash: String,
    embedding_id: Option<String>,
    collection_id: &'a CollectionId,
    ids: Vec<String>,
}

impl MemoryServiceImpl {
    fn build_vector_metadata(input: &ObservationInput) -> HashMap<String, serde_json::Value> {
        let mut vector_metadata = HashMap::new();
        vector_metadata.insert(
            METADATA_KEY_CONTENT.to_owned(),
            serde_json::Value::String(input.content.clone()),
        );
        vector_metadata.insert(
            METADATA_KEY_TYPE.to_owned(),
            serde_json::Value::String(input.r#type.as_str().to_owned()),
        );
        vector_metadata.insert(METADATA_KEY_TAGS.to_owned(), serde_json::json!(input.tags));
        vector_metadata.insert(
            "project_id".to_owned(),
            serde_json::Value::String(input.project_id.clone()),
        );

        if let Some(session_id) = &input.metadata.session_id {
            vector_metadata.insert(
                METADATA_KEY_SESSION_ID.to_owned(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        vector_metadata.insert(
            METADATA_KEY_FILE_PATH.to_owned(),
            serde_json::Value::String(
                input
                    .metadata
                    .file_path
                    .clone()
                    .unwrap_or_else(|| "memory".to_owned()),
            ),
        );
        vector_metadata.insert(
            METADATA_KEY_START_LINE.to_owned(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );

        vector_metadata
    }

    async fn insert_into_vector_store(
        &self,
        input: &ObservationInput,
        collection_id: &CollectionId,
    ) -> Result<Vec<String>> {
        let embedding = self.embedding_provider.embed(&input.content).await?;
        let vector_metadata = Self::build_vector_metadata(input);

        self.vector_store
            .insert_vectors(collection_id, &[embedding], vec![vector_metadata])
            .await
    }

    async fn create_and_store_observation(
        &self,
        input: ObservationInput,
        vector: VectorWriteOutcome<'_>,
    ) -> Result<String> {
        let observation = Observation {
            id: id::generate().to_string(),
            project_id: input.project_id,
            content: input.content,
            content_hash: vector.content_hash,
            tags: input.tags,
            r#type: input.r#type,
            metadata: input.metadata,
            created_at: domain_time::epoch_secs_i64()?,
            embedding_id: vector.embedding_id,
        };

        if let Err(err) = self.repository.store_observation(&observation).await {
            let _ = self
                .vector_store
                .delete_vectors(vector.collection_id, &vector.ids)
                .await;
            return Err(err);
        }

        Ok(observation.id)
    }

    pub(crate) async fn store_observation_impl(
        &self,
        input: ObservationInput,
    ) -> Result<(String, bool)> {
        if input.project_id.trim().is_empty() {
            return Err(mcb_domain::error::Error::invalid_argument(
                "Project ID cannot be empty for memory storage",
            ));
        }

        let content_hash = compute_content_hash(&input.content);

        if let Some(existing) = self.repository.find_by_hash(&content_hash).await? {
            return Ok((existing.id, true));
        }

        let collection_id =
            CollectionId::from_uuid(id::deterministic("collection", MEMORY_COLLECTION_NAME));

        let ids = self
            .insert_into_vector_store(&input, &collection_id)
            .await?;
        let vector = VectorWriteOutcome {
            content_hash,
            embedding_id: ids.first().cloned(),
            collection_id: &collection_id,
            ids,
        };

        let id = self.create_and_store_observation(input, vector).await?;

        Ok((id, false))
    }
}
