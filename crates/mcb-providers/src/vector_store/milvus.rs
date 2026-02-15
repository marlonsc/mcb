//! Milvus vector store provider implementation
//!
//! High-performance cloud vector database using Milvus.
//! Supports production-scale vector storage with automatic indexing and distributed search.
//!
//! # Architecture Violation (REF003)
//! Missing corresponding test file in `tests/` directory.
//!
// TODO(QUAL005): Pending (REF003): Create test file `crates/mcb-providers/tests/milvus_test.rs`.

use std::borrow::Cow;
use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::proto::schema::DataType;
use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use milvus::value::{Value, ValueVec};

use crate::constants::{
    MILVUS_DEFAULT_TIMEOUT_SECS, MILVUS_FIELD_VARCHAR_MAX_LENGTH, MILVUS_IVFFLAT_NLIST,
    MILVUS_METADATA_VARCHAR_MAX_LENGTH, MILVUS_QUERY_BATCH_SIZE,
};
use crate::provider_utils::{RetryConfig, retry_with_backoff};

/// Milvus vector store provider implementation
pub struct MilvusVectorStoreProvider {
    client: Client,
}

struct InsertPayload {
    expected_dims: usize,
    vectors_flat: Vec<f32>,
    file_paths: Vec<String>,
    start_lines: Vec<i64>,
    contents: Vec<String>,
}

impl MilvusVectorStoreProvider {
    /// Helper method to convert Milvus errors to domain errors
    fn map_milvus_error<T, E: std::fmt::Display>(
        result: std::result::Result<T, E>,
        operation: &str,
    ) -> Result<T> {
        result.map_err(|e| Error::vector_db(format!("Failed to {operation}: {e}")))
    }

    /// Create a new Milvus vector store provider
    ///
    /// # Arguments
    /// * `address` - Milvus server address (e.g., "<http://localhost:19530>")
    /// * `token` - Optional authentication token
    /// * `timeout_secs` - Connection timeout in seconds (default: 10)
    pub async fn new(
        address: String,
        _token: Option<String>,
        timeout_secs: Option<u64>,
    ) -> Result<Self> {
        // Ensure the address has a scheme (required by tonic transport)
        let endpoint = if address.starts_with("http://") || address.starts_with("https://") {
            address
        } else {
            format!("http://{address}")
        };

        let timeout = timeout_secs.unwrap_or(MILVUS_DEFAULT_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout);

        let client = tokio::time::timeout(timeout_duration, Client::new(endpoint.clone()))
            .await
            .map_err(|_| {
                Error::vector_db(format!(
                    "Milvus connection timed out after {timeout} seconds"
                ))
            })?
            .map_err(|e| {
                Error::vector_db(format!("Failed to connect to Milvus at {endpoint}: {e}"))
            })?;

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorStoreAdmin for MilvusVectorStoreProvider {
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let name_str = name.to_string();
        Self::map_milvus_error(
            self.client.has_collection(&name_str).await,
            "check collection",
        )
    }

    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let stats = self
            .client
            .get_collection_stats(&collection.to_string())
            .await
            .map_err(|e| {
                Error::vector_db(format!(
                    "Failed to get stats for collection '{collection}': {e}"
                ))
            })?;

        let mut result = HashMap::new();
        result.insert("collection".to_owned(), serde_json::json!(collection));
        result.insert("status".to_owned(), serde_json::json!("active"));

        if let Some(count_str) = stats.get("row_count")
            && let Ok(count) = count_str.parse::<i64>()
        {
            result.insert("vectors_count".to_owned(), serde_json::json!(count));
        }

        result.insert("provider".to_owned(), serde_json::json!("milvus"));
        Ok(result)
    }

    async fn flush(&self, collection: &CollectionId) -> Result<()> {
        let name_str = collection.to_string();
        let result = retry_with_backoff(
            RetryConfig::new(3, std::time::Duration::from_millis(1000)),
            |_| self.client.flush_collections(vec![&name_str]),
            |e| {
                let err_str = e.to_string();
                err_str.contains("RateLimit") || err_str.contains("rate limit")
            },
        )
        .await;

        result.map(|_| ()).map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("RateLimit") || err_str.contains("rate limit") {
                Error::vector_db(format!("Failed to flush collection after retries: {e}"))
            } else {
                Error::vector_db(format!("Failed to flush collection: {e}"))
            }
        })
    }

    fn provider_name(&self) -> &str {
        "milvus"
    }

    async fn health_check(&self) -> Result<()> {
        // Try to list collections as a health check - lighter than creating a test collection
        Self::map_milvus_error(
            self.client.list_collections().await,
            "health check (list collections)",
        )?;
        Ok(())
    }
}

impl MilvusVectorStoreProvider {
    /// Validate search parameters
    fn validate_search_params(query_vector: &[f32], limit: usize) -> Result<()> {
        if query_vector.is_empty() {
            return Err(Error::vector_db("Query vector cannot be empty".to_owned()));
        }
        if limit == 0 {
            return Err(Error::vector_db("Limit must be greater than 0".to_owned()));
        }
        Ok(())
    }

    /// Load collection with graceful error handling
    async fn load_collection_safe(&self, collection: &CollectionId) -> Result<()> {
        let name_str = collection.to_string();
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if err_str.contains("CollectionNotExists")
                || err_str.contains("collection not found")
                || err_str.contains("not exist")
            {
                tracing::debug!(
                    "Collection '{}' does not exist, returning empty results",
                    collection
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
    async fn perform_search(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<milvus::collection::SearchResult<'_>>> {
        use milvus::query::SearchOptions;
        let name_str = collection.to_string();

        let search_options = SearchOptions::new()
            .limit(limit)
            .output_fields(vec![
                "id".to_owned(),
                "file_path".to_owned(),
                "start_line".to_owned(),
                "content".to_owned(),
            ])
            .add_param("metric_type", "L2");

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

    fn extract_string_field(fields: &[FieldColumn], name: &str, index: usize) -> String {
        fields
            .iter()
            .find(|column| column.name == name)
            .and_then(|column| column.get(index))
            .and_then(|value| match value {
                Value::String(text) => Some(text.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| {
                if name == "content" {
                    String::new()
                } else {
                    "unknown".to_owned()
                }
            })
    }

    fn extract_long_field(fields: &[FieldColumn], name: &str, index: usize) -> i64 {
        fields
            .iter()
            .find(|column| column.name == name)
            .and_then(|column| column.get(index))
            .and_then(|value| match value {
                Value::Long(number) => Some(number),
                _ => None,
            })
            .unwrap_or(0)
    }

    fn build_collection_schema(name: &CollectionId, dimensions: usize) -> Result<CollectionSchema> {
        CollectionSchemaBuilder::new(&name.to_string(), &format!("Collection for {name}"))
            .add_field(FieldSchema::new_primary_int64(
                "id",
                "primary key field",
                true,
            ))
            .add_field(FieldSchema::new_float_vector(
                "vector",
                "feature field",
                dimensions as i64,
            ))
            .add_field(FieldSchema::new_varchar(
                "file_path",
                "file path",
                MILVUS_FIELD_VARCHAR_MAX_LENGTH,
            ))
            .add_field(FieldSchema::new_int64("start_line", "start line"))
            .add_field(FieldSchema::new_varchar(
                "content",
                "content",
                MILVUS_METADATA_VARCHAR_MAX_LENGTH,
            ))
            .build()
            .map_err(|e| Error::vector_db(format!("Failed to create schema: {e}")))
    }

    async fn create_vector_index_with_retry(&self, name: &CollectionId) -> Result<()> {
        use milvus::index::{IndexParams, IndexType, MetricType};
        let name_str = name.to_string();

        let index_result = retry_with_backoff(
            RetryConfig::new(3, std::time::Duration::from_millis(500)),
            |_| async {
                let index_params = IndexParams::new(
                    "vector_index".to_owned(),
                    IndexType::IvfFlat,
                    MetricType::L2,
                    HashMap::from([("nlist".to_owned(), MILVUS_IVFFLAT_NLIST.to_string())]),
                );
                self.client
                    .create_index(&name_str, "vector", index_params)
                    .await
            },
            |e| {
                let err_str = e.to_string();
                err_str.contains("CollectionNotExists") || err_str.contains("collection not found")
            },
        )
        .await;

        if let Err(e) = index_result {
            let err_str = e.to_string();
            if err_str.contains("CollectionNotExists") || err_str.contains("collection not found") {
                return Err(Error::vector_db(format!(
                    "Failed to create index after retries: {e}"
                )));
            }
            return Err(Error::vector_db(format!("Failed to create index: {e}")));
        }

        Ok(())
    }

    fn validate_insert_input(vectors: &[Embedding], metadata_len: usize) -> Result<usize> {
        if vectors.is_empty() {
            return Err(Error::vector_db(
                "No vectors provided for insertion".to_owned(),
            ));
        }

        if vectors.len() != metadata_len {
            return Err(Error::vector_db(format!(
                "Vectors ({}) and metadata ({}) arrays must have the same length",
                vectors.len(),
                metadata_len
            )));
        }

        let expected_dims = vectors[0].dimensions;
        for (i, vector) in vectors.iter().enumerate() {
            if vector.dimensions != expected_dims {
                return Err(Error::vector_db(format!(
                    "Vector at index {} has dimensions {} but expected {}",
                    i, vector.dimensions, expected_dims
                )));
            }
        }

        Ok(expected_dims)
    }

    fn prepare_insert_data(
        vectors: &[Embedding],
        metadata: &[HashMap<String, serde_json::Value>],
        expected_dims: usize,
    ) -> InsertPayload {
        let capacity = vectors.len();
        let mut payload = InsertPayload {
            expected_dims,
            vectors_flat: Vec::with_capacity(capacity * expected_dims),
            file_paths: Vec::with_capacity(capacity),
            start_lines: Vec::with_capacity(capacity),
            contents: Vec::with_capacity(capacity),
        };

        for (embedding, meta) in vectors.iter().zip(metadata.iter()) {
            payload.vectors_flat.extend_from_slice(&embedding.vector);
            payload.file_paths.push(
                meta.get("file_path")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_owned(),
            );
            payload.start_lines.push(
                meta.get("start_line")
                    .and_then(serde_json::Value::as_i64)
                    .or_else(|| meta.get("line_number").and_then(serde_json::Value::as_i64))
                    .unwrap_or(0),
            );
            payload.contents.push(
                meta.get("content")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_owned(),
            );
        }

        payload
    }

    fn build_field_column(
        name: &str,
        dtype: DataType,
        value: ValueVec,
        max_length: i32,
    ) -> FieldColumn {
        FieldColumn {
            name: name.to_owned(),
            dtype,
            value,
            dim: 1,
            max_length,
            is_dynamic: false,
        }
    }

    fn build_insert_columns(payload: InsertPayload) -> Vec<FieldColumn> {
        let vector_column = FieldColumn {
            name: "vector".to_owned(),
            dtype: DataType::FloatVector,
            value: ValueVec::Float(payload.vectors_flat),
            dim: payload.expected_dims as i64,
            max_length: 0,
            is_dynamic: false,
        };

        vec![
            vector_column,
            Self::build_field_column(
                "file_path",
                DataType::VarChar,
                ValueVec::String(payload.file_paths),
                MILVUS_FIELD_VARCHAR_MAX_LENGTH,
            ),
            Self::build_field_column(
                "start_line",
                DataType::Int64,
                ValueVec::Long(payload.start_lines),
                0,
            ),
            Self::build_field_column(
                "content",
                DataType::VarChar,
                ValueVec::String(payload.contents),
                MILVUS_METADATA_VARCHAR_MAX_LENGTH,
            ),
        ]
    }

    fn parse_milvus_ids(result: &milvus::proto::milvus::MutationResult) -> Vec<String> {
        match &result.i_ds {
            Some(ids) => match &ids.id_field {
                Some(milvus::proto::schema::i_ds::IdField::IntId(int_ids)) => int_ids
                    .data
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
                Some(milvus::proto::schema::i_ds::IdField::StrId(str_ids)) => str_ids.data.clone(),
                None => Vec::new(),
            },
            None => Vec::new(),
        }
    }

    fn convert_search_results(
        search_results: &[milvus::collection::SearchResult<'_>],
    ) -> Vec<SearchResult> {
        search_results
            .iter()
            .flat_map(|search_result| {
                search_result
                    .id
                    .iter()
                    .enumerate()
                    .map(|(index, id_value)| {
                        let distance_squared =
                            search_result.score.get(index).copied().unwrap_or(0.0);
                        let score = 1.0 / (1.0 + distance_squared.sqrt());
                        let fields = &search_result.field;
                        let start_line = Self::extract_long_field(fields, "start_line", index)
                            .max(Self::extract_long_field(fields, "line_number", index))
                            as u32;

                        SearchResult {
                            id: Self::value_to_id_string(Some(id_value.clone())),
                            file_path: Self::extract_string_field(fields, "file_path", index),
                            start_line,
                            content: Self::extract_string_field(fields, "content", index),
                            score: score as f64,
                            language: "unknown".to_owned(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn query_row_count(query_results: &[FieldColumn]) -> usize {
        query_results.first().map_or(0, FieldColumn::len)
    }

    fn convert_query_results(
        query_results: &[FieldColumn],
        file_path_override: Option<&str>,
    ) -> Vec<SearchResult> {
        (0..Self::query_row_count(query_results))
            .map(|index| {
                let file_path = file_path_override.map_or_else(
                    || Self::extract_string_field(query_results, "file_path", index),
                    std::string::ToString::to_string,
                );
                let start_line = Self::extract_long_field(query_results, "start_line", index).max(
                    Self::extract_long_field(query_results, "line_number", index),
                ) as u32;

                SearchResult {
                    id: Self::value_to_id_string(
                        query_results
                            .iter()
                            .find(|column| column.name == "id")
                            .and_then(|column| column.get(index)),
                    ),
                    file_path,
                    start_line,
                    content: Self::extract_string_field(query_results, "content", index),
                    score: 1.0,
                    language: "unknown".to_owned(),
                }
            })
            .collect()
    }

    async fn fetch_list_vectors_batch(
        &self,
        collection: &CollectionId,
        offset: i64,
        remaining: usize,
        current_total: usize,
    ) -> Result<Option<Vec<FieldColumn>>> {
        use milvus::query::QueryOptions;

        let batch_limit = remaining.min(MILVUS_QUERY_BATCH_SIZE) as i64;
        let query_options = QueryOptions::new()
            .limit(batch_limit)
            .offset(offset)
            .output_fields(vec![
                "id".to_owned(),
                "file_path".to_owned(),
                "start_line".to_owned(),
                "content".to_owned(),
            ]);

        match self
            .client
            .query(collection.to_string(), "id >= 0", &query_options)
            .await
        {
            Ok(results) => Ok(Some(results)),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("message length too large") {
                    tracing::warn!(
                        offset = offset,
                        results = current_total,
                        "hit gRPC message size limit, returning partial results"
                    );
                    return Ok(None);
                }
                Err(Error::vector_db(format!("Failed to list vectors: {e}")))
            }
        }
    }

    fn convert_to_file_infos(query_results: &[FieldColumn], limit: usize) -> Vec<FileInfo> {
        let mut file_counts: HashMap<String, u32> = HashMap::new();

        for index in 0..Self::query_row_count(query_results) {
            let path = Self::extract_string_field(query_results, "file_path", index);
            if path != "unknown" {
                *file_counts.entry(path).or_insert(0) += 1;
            }
        }

        file_counts
            .into_iter()
            .take(limit)
            .map(|(path, chunk_count)| FileInfo::new(path, chunk_count, "unknown", None))
            .collect()
    }
}

#[async_trait]
impl VectorStoreProvider for MilvusVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let schema = Self::build_collection_schema(name, dimensions)?;

        Self::map_milvus_error(
            self.client.create_collection(schema, None).await,
            "create collection",
        )?;

        // Wait for Milvus to sync collection metadata
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        self.create_vector_index_with_retry(name).await?;

        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let name_str = name.to_string();
        Self::map_milvus_error(
            self.client.drop_collection(&name_str).await,
            "delete collection",
        )?;
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let expected_dims = Self::validate_insert_input(vectors, metadata.len())?;
        let payload = Self::prepare_insert_data(vectors, &metadata, expected_dims);
        let columns = Self::build_insert_columns(payload);
        let name_str = collection.to_string();

        let res = Self::map_milvus_error(
            self.client.insert(&name_str, columns, None).await,
            "insert vectors",
        )?;

        Ok(Self::parse_milvus_ids(&res))
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Validate parameters
        Self::validate_search_params(query_vector, limit)?;

        // Load collection safely
        self.load_collection_safe(collection).await?;

        // Perform search
        let search_results = self.perform_search(collection, query_vector, limit).await?;

        // Convert results to our format
        Ok(Self::convert_search_results(&search_results))
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        use milvus::mutate::DeleteOptions;
        use milvus::value::ValueVec;

        // Convert string IDs to i64 for Milvus
        let id_numbers: Vec<i64> = ids.iter().filter_map(|id| id.parse::<i64>().ok()).collect();

        if id_numbers.is_empty() {
            return Ok(()); // Nothing to delete
        }

        let options = DeleteOptions::with_ids(ValueVec::Long(id_numbers));
        let name_str = collection.to_string();

        Self::map_milvus_error(
            self.client.delete(&name_str, &options).await,
            "delete vectors",
        )?;

        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let name_str = collection.to_string();

        // Ensure collection is loaded
        self.client
            .load_collection(&name_str, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{collection}': {e}"))
            })?;

        // Construct expression for query
        let expr = format!("id in [{}]", ids.join(","));

        use milvus::query::QueryOptions;
        let mut query_options = QueryOptions::new();
        query_options = query_options.output_fields(vec![
            "id".to_owned(),
            "file_path".to_owned(),
            "start_line".to_owned(),
            "content".to_owned(),
        ]);

        let query_results = Self::map_milvus_error(
            self.client.query(&name_str, &expr, &query_options).await,
            "query by IDs",
        )?;

        Ok(Self::convert_query_results(&query_results, None))
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let name_str = collection.to_string();

        // Ensure collection is loaded
        self.client
            .load_collection(&name_str, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{collection}': {e}"))
            })?;

        let mut all_results = Vec::new();
        let mut offset = 0i64;

        loop {
            let remaining = limit.saturating_sub(all_results.len());
            if remaining == 0 {
                break;
            }

            let Some(query_results) = self
                .fetch_list_vectors_batch(collection, offset, remaining, all_results.len())
                .await?
            else {
                break;
            };

            let row_count = Self::query_row_count(&query_results);
            if row_count == 0 {
                break;
            }

            all_results.extend(Self::convert_query_results(&query_results, None));

            offset += row_count as i64;

            if row_count < remaining.min(MILVUS_QUERY_BATCH_SIZE) {
                break;
            }
        }

        Ok(all_results)
    }
}

#[async_trait]
impl VectorStoreBrowser for MilvusVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collection_names =
            Self::map_milvus_error(self.client.list_collections().await, "list collections")?;

        let mut collections = Vec::new();

        for name in collection_names {
            let collection_id = CollectionId::from_name(&name);
            // Get stats for each collection
            let stats = self.get_stats(&collection_id).await.unwrap_or_default();
            let vector_count = stats
                .get("vectors_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

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
        let name_str = collection.to_string();

        // Ensure collection is loaded
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if err_str.contains("CollectionNotExists")
                || err_str.contains("collection not found")
                || err_str.contains("not exist")
            {
                return Ok(Vec::new());
            }
            return Err(Error::vector_db(format!(
                "Failed to load collection '{collection}': {e}"
            )));
        }

        use milvus::query::QueryOptions;

        let expr = "id >= 0".to_owned();
        let query_options = QueryOptions::new()
            .limit(crate::constants::MILVUS_DEFAULT_QUERY_LIMIT)
            .output_fields(vec!["file_path".to_owned()]);

        let query_results = match self.client.query(&name_str, &expr, &query_options).await {
            Ok(results) => results,
            Err(e) => {
                tracing::warn!(error = %e, "failed to query file paths");
                return Ok(Vec::new());
            }
        };

        Ok(Self::convert_to_file_infos(&query_results, limit))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let name_str = collection.to_string();
        // Ensure collection is loaded
        if let Err(e) = self.client.load_collection(&name_str, None).await {
            let err_str = e.to_string();
            if err_str.contains("CollectionNotExists")
                || err_str.contains("collection not found")
                || err_str.contains("not exist")
            {
                return Ok(Vec::new());
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
            .output_fields(vec![
                "id".to_owned(),
                "file_path".to_owned(),
                "start_line".to_owned(),
                "content".to_owned(),
            ]);

        let query_results = match self.client.query(&name_str, &expr, &query_options).await {
            Ok(results) => results,
            Err(e) => {
                tracing::warn!(error = %e, "failed to query chunks by file");
                return Ok(Vec::new());
            }
        };

        let mut results = Self::convert_query_results(&query_results, Some(file_path));
        results.sort_by_key(|r| r.start_line);

        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::registry::vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating Milvus vector store provider instances.
fn milvus_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let uri = config.uri.clone().ok_or_else(|| {
        format!(
            "Milvus requires 'uri' configuration (e.g., http://localhost:{})",
            crate::constants::MILVUS_DEFAULT_PORT
        )
    })?;
    let token = config.api_key.clone();

    // Create Milvus client synchronously using block_on
    let provider = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { MilvusVectorStoreProvider::new(uri, token, None).await })
    })
    .map_err(|e| format!("Failed to create Milvus provider: {e}"))?;

    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static MILVUS_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "milvus",
    description: "Milvus distributed vector database",
    factory: milvus_factory,
};
