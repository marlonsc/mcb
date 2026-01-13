//! Provider utilities - Provider identification and parsing (DRY)
//!
//! Consolidates provider type inference and parsing logic

/// Provider identification and parsing utilities
pub struct ProviderUtils;

impl ProviderUtils {
    /// Infer provider type from provider ID
    ///
    /// # Examples
    /// - "ollama" → "embedding"
    /// - "milvus" → "vector_store"
    /// - "unknown_provider" → "unknown"
    pub fn infer_type(provider_id: &str) -> &'static str {
        match provider_id.to_lowercase().as_str() {
            "ollama" | "openai" | "voyageai" | "gemini" | "fastembed" | "null_embedding" => {
                "embedding"
            }
            "milvus" | "edgevec" | "filesystem" | "encrypted" | "in_memory" | "null_vector" => {
                "vector_store"
            }
            _ => "unknown",
        }
    }

    /// Parse provider_id string into (type, id) tuple
    ///
    /// Supports two formats:
    /// - "embedding:ollama" → ("embedding", "ollama")
    /// - "ollama" → ("embedding", "ollama") - infers type
    ///
    /// # Examples
    /// - "embedding:custom_provider" → ("embedding", "custom_provider")
    /// - "vector_store:milvus_prod" → ("vector_store", "milvus_prod")
    /// - "milvus" → ("vector_store", "milvus")
    /// - "unknown_service" → ("unknown", "unknown_service")
    pub fn parse_provider_id(provider_id: &str) -> (String, String) {
        if provider_id.contains(':') {
            let parts: Vec<&str> = provider_id.splitn(2, ':').collect();
            (
                parts[0].to_string(),
                parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
            )
        } else {
            (
                Self::infer_type(provider_id).to_string(),
                provider_id.to_string(),
            )
        }
    }
}
