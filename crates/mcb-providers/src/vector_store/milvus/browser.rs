use super::*;
use async_trait::async_trait;
use mcb_domain::error::Error;
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};
use milvus::data::FieldColumn;
use std::collections::HashMap;

use crate::constants::{
    VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_ID, VECTOR_FIELD_START_LINE,
};
use schema::{extract_long_field, extract_string_field};

pub(super) fn query_row_count(query_results: &[FieldColumn]) -> usize {
    query_results.first().map_or(0, FieldColumn::len)
}

pub(super) fn convert_query_results(
    query_results: &[FieldColumn],
    file_path_override: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    for index in 0..query_row_count(query_results) {
        let file_path = match file_path_override {
            Some(path) => path.to_owned(),
            None => extract_string_field(query_results, VECTOR_FIELD_FILE_PATH, index)?,
        };
        let start_line = extract_long_field(query_results, VECTOR_FIELD_START_LINE, index)? as u32;

        results.push(SearchResult {
            id: value_to_id_string(
                query_results
                    .iter()
                    .find(|column| column.name == VECTOR_FIELD_ID)
                    .and_then(|column| column.get(index)),
            ),
            file_path,
            start_line,
            content: extract_string_field(query_results, VECTOR_FIELD_CONTENT, index)?,
            score: 1.0,
            language: "unknown".to_owned(),
        });
    }
    Ok(results)
}

fn value_to_id_string(value: Option<milvus::value::Value<'_>>) -> String {
    use milvus::value::Value;
    match value {
        Some(Value::Long(id)) => id.to_string(),
        Some(Value::String(id)) => id.to_string(),
        _ => "unknown".to_owned(),
    }
}

fn convert_to_file_infos(query_results: &[FieldColumn], limit: usize) -> Result<Vec<FileInfo>> {
    let mut file_counts: HashMap<String, u32> = HashMap::new();

    for index in 0..query_row_count(query_results) {
        let path = extract_string_field(query_results, VECTOR_FIELD_FILE_PATH, index)?;
        *file_counts.entry(path).or_insert(0) += 1;
    }

    Ok(file_counts
        .into_iter()
        .take(limit)
        .map(|(path, chunk_count)| FileInfo::new(path, chunk_count, "unknown", None))
        .collect())
}

#[async_trait]
impl VectorStoreBrowser for MilvusVectorStoreProvider {
    // --- Browser Methods ---

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collection_names =
            Self::map_milvus_error(self.client.list_collections().await, "list collections")?;

        let mut collections = Vec::new();

        for name in collection_names {
            let _collection_id = CollectionId::from_name(&name);
            // Get stats for each collection
            let stats = self.client.get_collection_stats(&name).await.map_err(|e| {
                Error::vector_db(format!("Failed to get stats for collection '{name}': {e}"))
            })?;
            let vector_count = stats
                .get("row_count")
                .and_then(|value: &String| value.parse::<u64>().ok())
                .ok_or_else(|| {
                    Error::vector_db(format!(
                        "Milvus collection '{name}' stats missing 'row_count'"
                    ))
                })?;

            // For now, we don't have a quick way to count unique files without querying all data
            // In a future optimization, we could cache this or use Milvus aggregation
            collections.push(CollectionInfo::new(
                name,
                vector_count,
                0, // file_count will be populated when listing files
                None,
                self.provider_name(),
            ));
        }

        Ok(collections)
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let name_str = to_milvus_name(collection);

        // Ensure collection is loaded
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if is_collection_not_found(&err_str) {
                return Err(Error::vector_db(format!(
                    "Collection '{collection}' not found when listing file paths"
                )));
            }
            return Err(Error::vector_db(format!(
                "Failed to load collection '{collection}': {e}"
            )));
        }

        use milvus::query::QueryOptions;

        let expr = "id >= 0".to_owned();
        let query_options = QueryOptions::new()
            .limit(crate::constants::MILVUS_DEFAULT_QUERY_LIMIT)
            .output_fields(vec![VECTOR_FIELD_FILE_PATH.to_owned()]);

        let query_results = match self.client.query(&name_str, &expr, &query_options).await {
            Ok(results) => results,
            Err(e) => {
                let msg = format!("Failed to query file paths in collection '{collection}': {e}");
                mcb_domain::warn!("milvus", "query file paths failed", &msg);
                return Err(Error::vector_db(msg));
            }
        };

        convert_to_file_infos(&query_results, limit)
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let name_str = to_milvus_name(collection);
        // Ensure collection is loaded
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if is_collection_not_found(&err_str) {
                return Err(Error::vector_db(format!(
                    "Collection '{collection}' not found when querying chunks by file"
                )));
            }
            return Err(Error::vector_db(format!(
                "Failed to load collection '{collection}': {e}"
            )));
        }

        use milvus::query::QueryOptions;

        // Query with filter on file_path
        let expr = format!("file_path == \"{}\"", file_path.replace('"', "\\\""));
        let query_options = QueryOptions::new()
            .limit(1000) // Reasonable limit for chunks per file
            .output_fields(Self::default_output_fields());

        let query_results = match self.client.query(&name_str, &expr, &query_options).await {
            Ok(results) => results,
            Err(e) => {
                let msg =
                    format!("Failed to query chunks by file in collection '{collection}': {e}");
                mcb_domain::warn!("milvus", "query chunks by file failed", &msg);
                return Err(Error::vector_db(msg));
            }
        };

        let mut results = convert_query_results(&query_results, Some(file_path))?;
        results.sort_by_key(|r| r.start_line);

        Ok(results)
    }
}
