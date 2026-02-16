//! Provider Constants
//!
//! Constants specific to provider implementations. These are separated from
//! domain constants (which live in mcb-domain) and infrastructure constants.

/// Cache constants (`Redis`, etc.)
pub mod cache;
/// Database constants (`SQL`, etc.)
pub mod database;
/// Embedding constants (`OpenAI`, `VoyageAI`, etc.)
pub mod embedding;
/// Event bus constants (`NATS`, `Tokio`)
pub mod events;
/// HTTP client constants
pub mod http;
/// Language processing constants
pub mod language;
/// Vector store constants (`Milvus`, `Qdrant`, etc.)
pub mod vector_store;

pub use self::cache::*;
pub use self::database::*;
pub use self::embedding::*;
pub use self::events::*;
pub use self::http::*;
pub use self::language::*;
pub use self::vector_store::*;

// ============================================================================
// Retry Configuration
// ============================================================================

/// Default retry count for embedding API requests.
pub const EMBEDDING_RETRY_COUNT: usize = 3;

/// Default retry backoff for embedding API requests (milliseconds).
pub const EMBEDDING_RETRY_BACKOFF_MS: u64 = 500;

/// Default retry count for vector store API requests.
pub const VECTOR_STORE_RETRY_COUNT: usize = 2;

/// Default retry backoff for vector store API requests (seconds).
pub const VECTOR_STORE_RETRY_BACKOFF_SECS: u64 = 1;
