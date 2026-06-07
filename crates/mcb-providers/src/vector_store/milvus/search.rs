use super::*;
use mcb_domain::error::Error;
use mcb_domain::value_objects::{CollectionId, SearchResult};
use milvus::value::Value;
use std::borrow::Cow;

use crate::constants::{
    MILVUS_DISTANCE_METRIC, MILVUS_PARAM_METRIC_TYPE, VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH,
    VECTOR_FIELD_START_LINE,
};
use schema::{extract_long_field, extract_string_field};

impl MilvusVectorStoreProvider {
    /// Validate search parameters
    pub(super) fn validate_search_params(query_vector: &[f32], limit: usize) -> Result<()> {
        if query_vector.is_empty() {
            return Err(Error::vector_db("Query vector cannot be empty".to_owned()));
        }
        if limit == 0 {
            return Err(Error::vector_db("Limit must be greater than 0".to_owned()));
        }
        Ok(())
    }

    /// Load collection with graceful error handling
    pub(super) async fn load_collection_safe(&self, collection: &CollectionId) -> Result<()> {
        let name_str = to_milvus_name(collection);
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if is_collection_not_found(&err_str) {
                mcb_domain::debug!(
                    "milvus",
                    "Collection does not exist, returning empty results"
                );
                return Err(Error::vector_db(format!(
                    "Collection '{collection}' not found"
                )));
            }
            return Err(Error::vector_db(format!(
                "Failed to load collection '{collection}': {e}"
            )));
        }
        Ok(())
    }

    /// Perform the actual search operation
    pub(super) async fn perform_search(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<milvus::collection::SearchResult<'_>>> {
        use milvus::query::SearchOptions;
        let name_str = to_milvus_name(collection);

        let search_options = SearchOptions::new()
            .limit(limit)
            .output_fields(Self::default_output_fields())
            .add_param(MILVUS_PARAM_METRIC_TYPE, MILVUS_DISTANCE_METRIC);

        self.client
            .search(
                &name_str,
                vec![Value::FloatArray(Cow::Borrowed(query_vector))],
                Some(search_options),
            )
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("no IDs") || err_str.contains("empty") {
                    Error::vector_db("No results found".to_owned())
                } else {
                    Error::vector_db(format!("Failed to search: {e}"))
                }
            })
    }

    fn value_to_id_string(value: Option<Value<'_>>) -> String {
        match value {
            Some(Value::Long(id)) => id.to_string(),
            Some(Value::String(id)) => id.to_string(),
            _ => "unknown".to_owned(),
        }
    }

    pub(super) fn convert_search_results(
        search_results: &[milvus::collection::SearchResult<'_>],
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        for search_result in search_results {
            for (index, id_value) in search_result.id.iter().enumerate() {
                let distance_squared =
                    search_result.score.get(index).copied().ok_or_else(|| {
                        Error::vector_db(format!(
                            "Milvus search result missing score at index {index}"
                        ))
                    })?;
                let score = 1.0 / (1.0 + distance_squared.sqrt());
                let fields = &search_result.field;
                let start_line = extract_long_field(fields, VECTOR_FIELD_START_LINE, index)? as u32;

                results.push(SearchResult {
                    id: Self::value_to_id_string(Some(id_value.clone())),
                    file_path: extract_string_field(fields, VECTOR_FIELD_FILE_PATH, index)?,
                    start_line,
                    content: extract_string_field(fields, VECTOR_FIELD_CONTENT, index)?,
                    score: score as f64,
                    language: "unknown".to_owned(),
                });
            }
        }
        Ok(results)
    }
}
