use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generates a new random UUID v4.
pub fn generate() -> Uuid {
    Uuid::new_v4()
}

/// Generates a deterministic UUID v5 from a namespace string and key.
pub fn deterministic(namespace: &str, key: &str) -> Uuid {
    let ns = Uuid::new_v5(&Uuid::NAMESPACE_OID, namespace.as_bytes());
    Uuid::new_v5(&ns, key.as_bytes())
}

/// Deterministic UUID v5 correlation string for a kind+raw_id pair.
///
/// Replaces the old HMAC-SHA256 `compute_stable_id_hash` / `hash_id` functions.
/// Same (kind, raw_id) always produces the same UUID string.
pub fn correlate_id(kind: &str, raw_id: &str) -> String {
    deterministic(kind, raw_id).to_string()
}

/// SHA-256 hex digest of content for deduplication.
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Mask sensitive ID for logging — shows first 8 chars + "...".
pub fn mask_id(id: &str) -> String {
    if id.len() <= 8 {
        id.to_string()
    } else {
        format!("{}...", &id[..8])
    }
}
