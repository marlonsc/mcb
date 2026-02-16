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
                        api_key,
                        base_url,
                        $default_base_url,
                        model,
                        timeout,
                        http_client,
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
            factory: $factory_fn,
        };
    };
}
