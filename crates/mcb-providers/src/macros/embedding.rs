//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#embedding-providers)
//!
//! HTTP embedding provider macros (struct definition, trait impl, registration).
//!
//! Used by embedding providers: openai, voyageai, anthropic, gemini, ollama.

#[macro_export]
/// Define a standard HTTP-based embedding provider struct.
///
/// Wraps `HttpEmbeddingClient` for shared functionality.
macro_rules! define_http_embedding_provider {
    (
        $(#[$meta:meta])*
        $struct_name:ident
    ) => {
        $(#[$meta])*
        pub struct $struct_name {
            client: HttpEmbeddingClient,
        }
    };
}

#[macro_export]
/// Implement standard base methods for HTTP embedding providers.
///
/// Adds `new()`, `base_url()`, and `model()` methods that delegate to `HttpEmbeddingClient`.
macro_rules! impl_http_provider_base {
    (
        $struct_name:ident,
        $default_base_url:expr
    ) => {
        impl $struct_name {
            /// Create a new provider instance
            #[must_use]
            pub fn new(
                api_key: &str,
                base_url: Option<String>,
                model: String,
                timeout: Duration,
                http_client: Client,
            ) -> Self {
                Self {
                    client: HttpEmbeddingClient::new(
                        $crate::utils::embedding::HttpEmbeddingClientConfig {
                            api_key: api_key.to_owned(),
                            base_url,
                            default_base_url: $default_base_url.to_owned(),
                            model,
                            timeout,
                            client: http_client,
                        },
                    ),
                }
            }

            /// Get the base URL for this provider
            #[must_use]
            pub fn base_url(&self) -> &str {
                &self.client.base_url
            }

            /// Get the model name
            #[must_use]
            pub fn model(&self) -> &str {
                &self.client.model
            }
        }
    };
}

#[macro_export]
/// Implement `EmbeddingProvider` trait for batch-capable HTTP providers.
///
/// Implements `embed_batch` using `process_batch` and generates standard provider metadata methods.
/// Requires the struct to implement `fetch_embeddings` and `parse_embedding`.
macro_rules! impl_embedding_provider_trait {
    (
        $struct_name:ident,
        $provider_slug:literal,
        $dimensions_logic:expr
    ) => {
        #[async_trait]
        impl EmbeddingProvider for $struct_name {
            async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
                process_batch(texts, self.fetch_embeddings(texts), |i, item| {
                    self.parse_embedding(i, item)
                })
                .await
            }

            fn dimensions(&self) -> usize {
                let model = self.client.model.as_str();
                let logic = ($dimensions_logic);
                logic(model)
            }

            fn provider_name(&self) -> &str {
                $provider_slug
            }
        }
    };
}

#[macro_export]
/// Register an HTTP embedding provider with linkme.
///
/// Generates the factory function and static registration entry for the provider registry.
macro_rules! register_http_provider {
    (
        $struct_name:ident,
        $factory_fn:ident,
        $static_name:ident,
        $provider_slug:literal,
        $description:literal,
        $config_name:literal,
        $default_model:literal
    ) => {
        /// Factory function for creating provider instances.
        fn $factory_fn(
            config: &EmbeddingProviderConfig,
        ) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
            use $crate::utils::http::create_http_provider_config;

            let cfg = create_http_provider_config(config, $config_name, $default_model)?;

            Ok(Arc::new($struct_name::new(
                &cfg.api_key,
                cfg.base_url,
                cfg.model,
                cfg.timeout,
                cfg.client,
            )))
        }

        #[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
        static $static_name: EmbeddingProviderEntry = EmbeddingProviderEntry {
            name: $provider_slug,
            description: $description,
            build: $factory_fn,
        };
    };
}

#[macro_export]
/// Generate a complete standard HTTP embedding provider.
///
/// This macro produces the entire provider implementation for providers that use
/// the standard `/v1/embeddings` API pattern with Bearer token authentication.
/// It generates: struct definition, `new()` + accessors, `fetch_embeddings()`,
/// `parse_embedding()`, `EmbeddingProvider` trait impl, and linkme registration.
///
/// # Parameters
///
/// - `struct_name`: Provider struct identifier
/// - `doc`: Doc comment for the struct
/// - `provider_name`: Display name for logging/errors (e.g. `"OpenAI"`)
/// - `provider_slug`: Registry slug (e.g. `"openai"`)
/// - `base_url`: Default base URL constant path
/// - `max_tokens`: Max tokens constant path
/// - `dimensions`: Closure `|model: &str| -> usize` for dimension lookup
/// - `extra_payload`: Optional extra JSON fields for the request payload
/// - `factory_fn`: Factory function identifier
/// - `static_name`: Linkme static identifier
/// - `description`: Human-readable description for the registry
/// - `config_name`: Config key name
/// - `default_model`: Default model name
///
/// # Example
///
/// ```rust,ignore
/// define_standard_embedding_provider! {
///     struct_name: OpenAIEmbeddingProvider,
///     doc: "OpenAI embedding provider",
///     provider_name: "OpenAI",
///     provider_slug: "openai",
///     base_url: mcb_utils::constants::embedding::OPENAI_API_BASE_URL,
///     max_tokens: mcb_utils::constants::embedding::OPENAI_MAX_TOKENS_PER_REQUEST,
///     dimensions: |model: &str| match model {
///         "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
///         "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
///         _ => EMBEDDING_DIMENSION_OPENAI_SMALL,
///     },
///     extra_payload: { "encoding_format": "float" },
///     factory_fn: openai_factory,
///     static_name: OPENAI_PROVIDER,
///     description: "OpenAI embedding provider (text-embedding-3-small/large, ada-002)",
///     config_name: "OpenAI",
///     default_model: "text-embedding-3-small",
/// }
/// ```
macro_rules! define_standard_embedding_provider {
    (
        struct_name: $struct_name:ident,
        doc: $doc:literal,
        provider_name: $provider_name:literal,
        provider_slug: $provider_slug:literal,
        base_url: $base_url:expr,
        max_tokens: $max_tokens:expr,
        dimensions: $dimensions_logic:expr,
        $(extra_payload: { $($extra_key:literal : $extra_val:literal),* $(,)? },)?
        factory_fn: $factory_fn:ident,
        static_name: $static_name:ident,
        description: $description:literal,
        config_name: $config_name:literal,
        default_model: $default_model:literal $(,)?
    ) => {
        // ── struct ──
        define_http_embedding_provider!(
            #[doc = $doc]
            $struct_name
        );

        // ── new() + accessors ──
        impl_http_provider_base!($struct_name, $base_url);

        impl $struct_name {
            /// Get the maximum tokens for this model
            #[must_use]
            pub fn max_tokens(&self) -> usize {
                $max_tokens
            }

            /// Send embedding request and get response data
            async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
                let payload = serde_json::json!({
                    (mcb_utils::constants::embedding::EMBEDDING_PARAM_INPUT): texts,
                    (mcb_utils::constants::embedding::EMBEDDING_PARAM_MODEL): self.client.model
                    $(, $($extra_key: $extra_val),*)?
                });

                let headers = vec![
                    (
                        mcb_utils::constants::http::HTTP_HEADER_AUTHORIZATION,
                        format!("Bearer {}", self.client.api_key),
                    ),
                    (
                        mcb_utils::constants::http::HTTP_HEADER_CONTENT_TYPE,
                        mcb_utils::constants::http::CONTENT_TYPE_JSON.to_owned(),
                    ),
                ];

                $crate::utils::http::send_json_request($crate::utils::http::JsonRequestParams {
                    client: &self.client.client,
                    method: reqwest::Method::POST,
                    url: format!(
                        "{}{}",
                        self.base_url(),
                        mcb_utils::constants::embedding::EMBEDDING_API_ENDPOINT,
                    ),
                    timeout: self.client.timeout,
                    provider: $provider_name,
                    operation: mcb_utils::constants::embedding::EMBEDDING_OPERATION_NAME,
                    kind: $crate::utils::http::RequestErrorKind::Embedding,
                    headers: &headers,
                    body: Some(&payload),
                    retry: Some($crate::utils::http::RetryConfig::new(
                        mcb_utils::constants::http::PROVIDER_RETRY_COUNT,
                        std::time::Duration::from_millis(
                            mcb_utils::constants::http::PROVIDER_RETRY_BACKOFF_MS,
                        ),
                    )),
                })
                .await
            }

            /// Parse embedding vector from response data
            fn parse_embedding(
                &self,
                index: usize,
                item: &serde_json::Value,
            ) -> Result<Embedding> {
                $crate::utils::embedding::parse_standard_embedding(
                    &self.client.model,
                    self.dimensions(),
                    index,
                    item,
                )
            }
        }

        // ── EmbeddingProvider trait ──
        impl_embedding_provider_trait!($struct_name, $provider_slug, $dimensions_logic);

        // ── linkme registration ──
        use std::sync::Arc;
        use mcb_domain::ports::EmbeddingProvider as EmbeddingProviderPort;
        use mcb_domain::registry::embedding::{
            EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
        };

        register_http_provider!(
            $struct_name,
            $factory_fn,
            $static_name,
            $provider_slug,
            $description,
            $config_name,
            $default_model
        );
    };
}
