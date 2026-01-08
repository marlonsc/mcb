use serde::{Deserialize, Serialize};

/// Embedding provider configuration types (similar to Claude Context)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum EmbeddingProviderConfig {
    #[serde(rename = "openai")]
    OpenAI {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "ollama")]
    Ollama {
        model: String,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "voyageai")]
    VoyageAI {
        model: String,
        api_key: String,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "gemini")]
    Gemini {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "mock")]
    Mock {
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "fastembed")]
    FastEmbed {
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
}
