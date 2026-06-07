use super::*;
use async_trait::async_trait;
use mcb_domain::error::Error;
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};
use std::collections::HashMap;

use crate::constants::{
    MILVUS_ERROR_COLLECTION_NOT_EXISTS, MILVUS_IVFFLAT_NLIST, MILVUS_PARAM_NLIST,
    MILVUS_VECTOR_INDEX_NAME, PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
};
use crate::utils::retry::{RetryConfig, retry_with_backoff};
use helpers::{build_insert_columns, parse_milvus_ids, prepare_insert_data, validate_insert_input};
use schema::build_collection_schema;

impl MilvusVectorStoreProvider {
    async fn create_vector_index_with_retry(&self, name: &CollectionId) -> Result<()> {
        use milvus::index::{IndexParams, IndexType, MetricType};
        let name_str = to_milvus_name(name);

        let index_result: std::result::Result<(), milvus::error::Error> = retry_with_backoff(
            RetryConfig::new(
                PROVIDER_RETRY_COUNT,
                std::time::Duration::from_millis(PROVIDER_RETRY_BACKOFF_MS),
            ),
            |_| async {
                let nlist_params: HashMap<String, String> = HashMap::from([(
                    MILVUS_PARAM_NLIST.to_owned(),
                    MILVUS_IVFFLAT_NLIST.to_string(),
                )]);
                let index_params = IndexParams::new(
                    MILVUS_VECTOR_INDEX_NAME.to_owned(),
                    IndexType::IvfFlat,
                    MetricType::L2,
                    nlist_params,
                );
                self.client
                    .create_index(
                        &name_str,
                        crate::constants::VECTOR_FIELD_VECTOR,
                        index_params,
                    )
                    .await
            },
            |e| {
                let err_str = e.to_string();
                err_str.contains(MILVUS_ERROR_COLLECTION_NOT_EXISTS)
                    || err_str.contains("collection not found")
            },
        )
        .await;

        if let Err(e) = index_result {
            let err_str = e.to_string();
            if err_str.contains(MILVUS_ERROR_COLLECTION_NOT_EXISTS)
                || err_str.contains("collection not found")
            {
                return Err(Error::vector_db(format!(
                    "Failed to create index after retries: {e}"
                )));
            }
            return Err(Error::vector_db(format!("Failed to create index: {e}")));
        }

        Ok(())
    }
}

#[async_trait]
impl VectorStoreProvider for MilvusVectorStoreProvider {
    async fn create_collection(&self, name: &CollectionId, dimensions: usize) -> Result<()> {
        let schema = build_collection_schema(name, dimensions)?;
        Self::map_milvus_error(
            self.client.create_collection(schema, None).await,
            "create collection",
        )?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        self.create_vector_index_with_retry(name).await?;
        Ok(())
    }

    async fn delete_collection(&self, name: &CollectionId) -> Result<()> {
        let name_str = to_milvus_name(name);
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
        let expected_dims = validate_insert_input(vectors, metadata.len())?;
        let payload = prepare_insert_data(vectors, &metadata, expected_dims)?;
        let columns = build_insert_columns(payload);
        let name_str = to_milvus_name(collection);
        let res = Self::map_milvus_error(
            self.client.insert(&name_str, columns, None).await,
            "insert vectors",
        )?;
        Ok(parse_milvus_ids(&res)?)
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        Self::validate_search_params(query_vector, limit)?;
        self.load_collection_safe(collection).await?;
        let search_results = self.perform_search(collection, query_vector, limit).await?;
        Self::convert_search_results(&search_results)
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        use milvus::mutate::DeleteOptions;
        use milvus::value::ValueVec;

        let id_numbers: Vec<i64> = ids.iter().filter_map(|id| id.parse::<i64>().ok()).collect();
        if id_numbers.is_empty() {
            return Ok(());
        }

        let options = DeleteOptions::with_ids(ValueVec::Long(id_numbers));
        let name_str = to_milvus_name(collection);
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
        let name_str = to_milvus_name(collection);
        self.client
            .load_collection(&name_str, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{collection}': {e}"))
            })?;

        let expr = format!("id in [{}]", ids.join(","));
        use milvus::query::QueryOptions;
        let query_options = QueryOptions::new().output_fields(Self::default_output_fields());
        let query_results = Self::map_milvus_error(
            self.client.query(&name_str, &expr, &query_options).await,
            "query by IDs",
        )?;
        browser::convert_query_results(&query_results, None)
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.list_vectors_impl(collection, limit).await
    }
}
