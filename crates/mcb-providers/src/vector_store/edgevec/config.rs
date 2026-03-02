use super::*;

/// `EdgeVec` vector store configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct EdgeVecConfig {
    /// Vector dimensionality
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,

    /// HNSW parameters for index optimization
    #[serde(default)]
    pub hnsw_config: HnswConfig,

    /// Distance metric to use
    #[serde(default)]
    pub metric: MetricType,

    /// Whether to use quantization for memory optimization
    #[serde(default)]
    pub use_quantization: bool,

    /// Quantization configuration
    #[serde(default)]
    pub quantizer_config: QuantizerConfig,
}

fn default_dimensions() -> usize {
    EDGEVEC_DEFAULT_DIMENSIONS
}

/// HNSW configuration for `EdgeVec`
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct HnswConfig {
    /// Maximum connections per node in layers > 0
    #[serde(default = "default_m")]
    pub m: u32,

    /// Maximum connections per node in layer 0
    #[serde(default = "default_m0")]
    pub m0: u32,

    /// Construction-time candidate list size
    #[serde(default = "default_ef_construction")]
    pub ef_construction: u32,

    /// Search-time candidate list size
    #[serde(default = "default_ef_search")]
    pub ef_search: u32,
}

fn default_m() -> u32 {
    EDGEVEC_HNSW_M
}
fn default_m0() -> u32 {
    EDGEVEC_HNSW_M0
}
fn default_ef_construction() -> u32 {
    EDGEVEC_HNSW_EF_CONSTRUCTION
}
fn default_ef_search() -> u32 {
    EDGEVEC_HNSW_EF_SEARCH
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: default_m(),
            m0: default_m0(),
            ef_construction: default_ef_construction(),
            ef_search: default_ef_search(),
        }
    }
}

/// Distance metrics supported by `EdgeVec`
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub enum MetricType {
    /// L2 Squared (Euclidean) distance
    L2Squared,
    /// Cosine similarity
    #[default]
    Cosine,
    /// Dot product
    DotProduct,
}

/// Quantization configuration for memory optimization
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct QuantizerConfig {
    /// `EdgeVec` quantization type for scalar quantization.
    #[serde(default)]
    pub quantization_type: String,
}

impl Default for QuantizerConfig {
    fn default() -> Self {
        Self {
            quantization_type: EDGEVEC_QUANTIZATION_TYPE.to_owned(),
        }
    }
}

impl Default for EdgeVecConfig {
    fn default() -> Self {
        Self {
            dimensions: default_dimensions(),
            hnsw_config: HnswConfig::default(),
            metric: MetricType::default(),
            use_quantization: false,
            quantizer_config: QuantizerConfig::default(),
        }
    }
}
