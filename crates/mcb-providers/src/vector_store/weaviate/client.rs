//! Weaviate vector store client implementation.

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{CollectionId, SearchResult};
use reqwest::Client;
use serde_json::Value;

use mcb_utils::constants::http::{
    CONTENT_TYPE_JSON, HTTP_HEADER_AUTHORIZATION, HTTP_HEADER_CONTENT_TYPE,
    PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
};
use mcb_utils::constants::vector_store::{
    VECTOR_FIELD_CONTENT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LANGUAGE, VECTOR_FIELD_START_LINE,
    WEAVIATE_AUTH_SCHEME, WEAVIATE_CLASS_PREFIX,
};

use crate::utils::http::{VectorDbRequestParams, send_vector_db_request};
use crate::utils::vector_store::search_result_from_json_metadata;

/// Weaviate vector store provider.
///
/// Implements the vector store domain ports using Weaviate's REST + GraphQL API.
/// Collections map to Weaviate classes; vectors are app-supplied (`vectorizer: none`).
pub struct WeaviateVectorStoreProvider {
    pub(super) base_url: String,
    pub(super) api_key: Option<String>,
    pub(super) timeout: Duration,
    pub(super) http_client: Client,
    /// Track known collections locally with their dimensions.
    pub(super) collections: Arc<DashMap<String, usize>>,
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
    pub(super) fn class_name(collection: &CollectionId) -> String {
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
    pub(super) async fn request(
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
            retry_attempts: PROVIDER_RETRY_COUNT,
            retry_backoff_ms: PROVIDER_RETRY_BACKOFF_MS,
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
    pub(super) fn where_clause(filter: &str) -> Result<String> {
        let parsed: Value = serde_json::from_str(filter)
            .map_err(|e| Error::vector_db(format!("invalid filter JSON: {e}")))?;
        let obj = parsed
            .as_object()
            .ok_or_else(|| Error::vector_db("filter must be a JSON object".to_owned()))?;

        let mut operands = Vec::with_capacity(obj.len());
        for (field, cond) in obj {
            let eq = cond.get("$eq").ok_or_else(|| {
                Error::vector_db(format!(
                    "unsupported filter for '{field}': only $eq is supported"
                ))
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
    pub(super) fn build_get_query(
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
    pub(super) async fn run_get_query(
        &self,
        class: &str,
        query: String,
    ) -> Result<Vec<SearchResult>> {
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
            return Err(Error::vector_db(format!(
                "Weaviate GraphQL error: {errors:?}"
            )));
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
                    .unwrap_or_default()
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
