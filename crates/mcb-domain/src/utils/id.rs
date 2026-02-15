use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use uuid::Uuid;

/// Generates a new random UUID v4.
#[must_use]
pub fn generate() -> Uuid {
    Uuid::new_v4()
}

/// Generates a deterministic UUID v5 from a namespace string and key.
#[must_use]
pub fn deterministic(namespace: &str, key: &str) -> Uuid {
    let ns = Uuid::new_v5(&Uuid::NAMESPACE_OID, namespace.as_bytes());
    Uuid::new_v5(&ns, key.as_bytes())
}

/// Deterministic UUID v5 correlation string for a `kind+raw_id` pair.
///
/// Replaces the old HMAC-SHA256 `compute_stable_id_hash` / `hash_id` functions.
/// Same (kind, `raw_id`) always produces the same UUID string.
#[must_use]
pub fn correlate_id(kind: &str, raw_id: &str) -> String {
    deterministic(kind, raw_id).to_string()
}

/// SHA-256 hex digest of content for deduplication.
#[must_use]
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Mask sensitive ID for logging — shows first 8 chars + "...".
#[must_use]
pub fn mask_id(id: &str) -> String {
    if id.len() <= 8 {
        id.to_owned()
    } else {
        format!("{}...", &id[..8])
    }
}

/// Compute SHA-256 hash of a file's content.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let file =
        File::open(path).map_err(|e| Error::io(format!("Failed to open file {path:?}: {e}")))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|e| Error::io(format!("Failed to read file {path:?}: {e}")))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}
