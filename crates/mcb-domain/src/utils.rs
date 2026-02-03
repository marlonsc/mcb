use sha2::{Digest, Sha256};

pub mod project_type;
pub mod submodule;
pub mod vcs_context;

/// SHA-256 hex digest of content for deduplication.
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
