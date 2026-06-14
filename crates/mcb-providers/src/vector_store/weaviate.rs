//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Weaviate Vector Store Provider
//!
//! Implements the vector store domain ports using Weaviate's REST + GraphQL API
//! (no official Rust SDK; communicates via the reqwest HTTP client).
//!
//! Collections map to Weaviate classes (vectorizer `none`, app-supplied vectors).
//! Vector search uses GraphQL `Get { Class(nearVector: ...) }`; object CRUD uses the
//! REST `/v1/objects` and `/v1/batch/objects` endpoints. Tenant isolation (ADR-056 A3)
//! is layered on top of this provider via Weaviate native multi-tenancy.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use reqwest::Client;
use serde_json::Value;

use crate::constants::{
    HTTP_HEADER_AUTHORIZATION, HTTP_HEADER_CONTENT_TYPE, STATS_FIELD_COLLECTION,
    STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT, STATUS_ACTIVE,
    STATUS_UNKNOWN, VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LANGUAGE,
    VECTOR_FIELD_START_LINE, VECTOR_STORE_RETRY_BACKOFF_SECS, VECTOR_STORE_RETRY_COUNT,
    WEAVIATE_AUTH_SCHEME, WEAVIATE_BATCH_SIZE, WEAVIATE_CLASS_PREFIX, WEAVIATE_DISTANCE_METRIC,
};
use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;

/// Weaviate vector store provider.
///
/// Implements the vector store domain ports using Weaviate's REST + GraphQL API.
pub struct WeaviateVectorStoreProvider {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    http_client: Client,
    /// Track known collections locally with their dimensions.
    collections: Arc<DashMap<String, usize>>,
}

impl WeaviateVectorStoreProvider {
    /// Create a new Weaviate vector store provider.
    ///
    /// # Arguments
    /// * `base_url` - Weaviate server base URL (e.g. `http://host:8080`)
    /// * `api_key` - optional API key (sent as a bearer token when present)
    /// * `timeout` - request timeout duration
    /// * `http_client` - reqwest HTTP client
    #[must_use]
    pub fn new(
        base_url: &str,
        api_key: Option<String>,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key: api_key.map(|k| k.trim().to_owned()),
            timeout,
            http_client,
            collections: Arc::new(DashMap::new()),
        }
    }

    /// Derive a valid Weaviate class name from a collection id.
    ///
    /// Weaviate classes must match `^[A-Z][_0-9A-Za-z]*$`, so non-alphanumeric
    /// characters are mapped to `_` and an uppercase prefix is prepended.
    fn class_name(collection: &CollectionId) -> String {
        let sanitized: String = collection
            .to_string()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("{WEAVIATE_CLASS_PREFIX}{sanitized}")
    }

    /// Build a full URL for a Weaviate API path.
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Make an authenticated request to Weaviate.
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value> {
        let mut headers = vec![(HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned())];
        if let Some(api_key) = &self.api_key {
            headers.push((
                HTTP_HEADER_AUTHORIZATION,
                format!("{WEAVIATE_AUTH_SCHEME} {api_key}"),
            ));
        }

        send_vector_db_request(VectorDbRequestParams {
            client: &self.http_client,
            method,
            url: self.api_url(path),
            timeout: self.timeout,
            provider: "Weaviate",
            operation: path,
            headers: &headers,
            body: body.as_ref(),
            retry_attempts: VECTOR_STORE_RETRY_COUNT,
            retry_backoff_secs: VECTOR_STORE_RETRY_BACKOFF_SECS,
        })
        .await
    }

    /// GraphQL field selection shared by all `Get` queries.
    fn graphql_fields() -> String {
        format!(
            "{VECTOR_FIELD_FILE_PATH} {VECTOR_FIELD_START_LINE} {VECTOR_FIELD_CONTENT} \
             {VECTOR_FIELD_LANGUAGE} _additional {{ id certainty distance }}"
        )
    }

    /// Translate a JSON `{field: {"$eq": "value"}}` filter into a Weaviate `where` clause.
    ///
    /// Only string equality (`$eq`) is supported; any other shape is a typed error
    /// rather than a silent no-op (no hidden failures).
    fn where_clause(filter: &str) -> Result<String> {
        let parsed: Value = serde_json::from_str(filter)
            .map_err(|e| Error::vector_db(format!("invalid filter JSON: {e}")))?;
        let obj = parsed
            .as_object()
            .ok_or_else(|| Error::vector_db("filter must be a JSON object".to_owned()))?;

        let mut operands = Vec::with_capacity(obj.len());
        for (field, cond) in obj {
            let eq = cond.get("$eq").ok_or_else(|| {
                Error::vector_db(format!("unsupported filter for '{field}': only $eq is supported"))
            })?;
            let value = eq
                .as_str()
                .ok_or_else(|| Error::vector_db("filter $eq value must be a string".to_owned()))?;
            let field_json = serde_json::to_string(field)
                .map_err(|e| Error::vector_db(format!("serialize filter field: {e}")))?;
            let value_json = serde_json::to_string(value)
                .map_err(|e| Error::vector_db(format!("serialize filter value: {e}")))?;
            operands.push(format!(
                "{{ path: [{field_json}], operator: Equal, valueText: {value_json} }}"
            ));
        }

        match operands.len() {
            0 => Err(Error::vector_db(
                "filter must contain at least one field".to_owned(),
            )),
            1 => Ok(operands.swap_remove(0)),
            _ => Ok(format!(
                "{{ operator: And, operands: [{}] }}",
                operands.join(", ")
            )),
        }
    }

    /// Build a GraphQL `Get` query with an optional `nearVector` and `where` clause.
    fn build_get_query(
        class: &str,
        vector: Option<&[f32]>,
        limit: usize,
        where_clause: Option<&str>,
    ) -> Result<String> {
        let mut args = vec![format!("limit: {limit}")];
        if let Some(v) = vector {
            let vector_json = serde_json::to_string(v)
                .map_err(|e| Error::vector_db(format!("serialize query vector: {e}")))?;
            args.push(format!("nearVector: {{ vector: {vector_json} }}"));
        }
        if let Some(w) = where_clause {
            args.push(format!("where: {w}"));
        }
        Ok(format!(
            "{{ Get {{ {class}({}) {{ {} }} }} }}",
            args.join(", "),
            Self::graphql_fields()
        ))
    }

    /// Execute a GraphQL query and parse `data.Get.<class>` into search results.
    async fn run_get_query(&self, class: &str, query: String) -> Result<Vec<SearchResult>> {
        let response = self
            .request(
                reqwest::Method::POST,
                "/v1/graphql",
                Some(serde_json::json!({ "query": query })),
            )
            .await?;

        if let Some(errors) = response.get("errors").and_then(Value::as_array)
            && !errors.is_empty()
        {
            return Err(Error::vector_db(format!("Weaviate GraphQL error: {errors:?}")));
        }

        let items = response
            .get("data")
            .and_then(|d| d.get("Get"))
            .and_then(|g| g.get(class))
            .and_then(Value::as_array)
            .ok_or_else(|| {
                Error::vector_db("Invalid Weaviate response: missing data.Get array".to_owned())
            })?;

        let results = items
            .iter()
            .map(|obj| {
                let additional = obj.get("_additional");
                let id = additional
                    .and_then(|a| a.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let score = additional
                    .and_then(|a| a.get("certainty"))
                    .and_then(Value::as_f64)
                    .or_else(|| {
                        additional
                            .and_then(|a| a.get("distance"))
                            .and_then(Value::as_f64)
                            .map(|d| 1.0 - d)
                    })
                    .unwrap_or(0.0);
                let mut metadata = obj.clone();
                if let Some(map) = metadata.as_object_mut() {
                    map.remove("_additional");
                }
                search_result_from_json_metadata(id, &metadata, score)
            })
            .collect();
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreAdmin for WeaviateVectorStoreProvider {
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let class = Self::class_name(name);
        match self
            .request(reqwest::Method::GET, &format!("/v1/schema/{class}"), None)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let class = Self::class_name(collection);
        let query = format!("{{ Aggregate {{ {class} {{ meta {{ count }} }} }} }}");

        let response = self
            .request(
                reqwest::Method::POST,
                "/v1/graphql",
                Some(serde_json::json!({ "query": query })),
            )
            .await;

        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        match response {
            Ok(data) => {
                let count = data
                    .get("data")
                    .and_then(|d| d.get("Aggregate"))
                    .and_then(|a| a.get(&class))
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.first())
                    .and_then(|m| m.get("meta"))
                    .and_then(|m| m.get("count"))
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!(0));
                stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), count);
                stats.insert(STATS_FIELD_STATUS.to_owned(), serde_json::json!(STATUS_ACTIVE));
            }
            Err(_) => {
                stats.insert(
                    STATS_FIELD_STATUS.to_owned(),
                    serde_json::json!(STATUS_UNKNOWN),
                );
                stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), serde_json::json!(0));
            }
        }
        Ok(stats)
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        // Weaviate writes are durable/consistent once acknowledged.
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "weaviate"
    }
}

#[async_trait]
impl VectorStoreBrowser for WeaviateVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self
            .request(reqwest::Method::GET, "/v1/schema", None)
            .await?;

        let classes = response
            .get("classes")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.get("class").and_then(Value::as_str))
                    .map(|name| CollectionInfo::new(name, 0, 0, None, self.provider_name()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(classes)
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        let results = self.list_vectors(collection, limit).await?;
        Ok(crate::utils::vector_store::build_file_info_from_results(
            results,
        ))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let class = Self::class_name(collection);
        let filter = serde_json::json!({ (VECTOR_FIELD_FILE_PATH): { "$eq": file_path } });
        let where_clause = Self::where_clause(&filter.to_string())?;
        let query = Self::build_get_query(&class, None, WEAVIATE_BATCH_SIZE, Some(&where_clause))?;

        let mut results = self.run_get_query(&class, query).await?;
        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreProvider for WeaviateVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let name_str = name.to_string();
        if self.collections.contains_key(&name_str) {
            return Err(Error::vector_db(format!(
                "Collection '{name}' already exists"
            )));
        }
        let class = Self::class_name(name);
        let body = serde_json::json!({
            "class": class,
            "vectorizer": "none",
            "vectorIndexConfig": { "distance": WEAVIATE_DISTANCE_METRIC },
        });
        self.request(reqwest::Method::POST, "/v1/schema", Some(body))
            .await?;
        self.collections.insert(name_str, dimensions);
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let class = Self::class_name(name);
        self.request(reqwest::Method::DELETE, &format!("/v1/schema/{class}"), None)
            .await?;
        self.collections.remove(&name.to_string());
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }
        if vectors.len() != metadata.len() {
            return Err(Error::vector_db(format!(
                "vectors/metadata length mismatch: {} vs {}",
                vectors.len(),
                metadata.len()
            )));
        }
        let class = Self::class_name(collection);

        let mut ids = Vec::with_capacity(vectors.len());
        let mut objects = Vec::with_capacity(vectors.len());

        for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let object_id = id::generate().to_string();
            objects.push(serde_json::json!({
                "class": class,
                "id": object_id,
                "vector": embedding.vector,
                "properties": meta,
            }));
            ids.push(object_id);

            if objects.len() >= WEAVIATE_BATCH_SIZE || i == vectors.len() - 1 {
                self.request(
                    reqwest::Method::POST,
                    "/v1/batch/objects",
                    Some(serde_json::json!({ "objects": objects })),
                )
                .await?;
                objects.clear();
            }
        }
        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let class = Self::class_name(collection);
        let where_clause = match filter {
            Some(f) => Some(Self::where_clause(f)?),
            None => None,
        };
        let query =
            Self::build_get_query(&class, Some(query_vector), limit, where_clause.as_deref())?;
        self.run_get_query(&class, query).await
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        let class = Self::class_name(collection);
        for object_id in ids {
            self.request(
                reqwest::Method::DELETE,
                &format!("/v1/objects/{class}/{object_id}"),
                None,
            )
            .await?;
        }
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
        let class = Self::class_name(collection);
        let mut results = Vec::with_capacity(ids.len());
        for object_id in ids {
            let response = self
                .request(
                    reqwest::Method::GET,
                    &format!("/v1/objects/{class}/{object_id}"),
                    None,
                )
                .await?;
            let empty = serde_json::Value::Object(Default::default());
            let properties = response.get("properties").unwrap_or(&empty);
            let resolved_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(object_id)
                .to_owned();
            results.push(search_result_from_json_metadata(resolved_id, properties, 1.0));
        }
        Ok(results)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let class = Self::class_name(collection);
        let query = Self::build_get_query(&class, None, limit, None)?;
        self.run_get_query(&class, query).await
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_domain::registry::vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating Weaviate vector store provider instances.
fn weaviate_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let uri = config
        .uri
        .clone()
        .ok_or_else(|| "Weaviate requires uri (http://host:8080)".to_owned())?;
    let http_client = create_default_client()?;

    Ok(Arc::new(WeaviateVectorStoreProvider::new(
        &uri,
        config.api_key.clone(),
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static WEAVIATE_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "weaviate",
    description: "Weaviate vector database (REST + GraphQL, native multi-tenancy)",
    build: weaviate_factory,
};
