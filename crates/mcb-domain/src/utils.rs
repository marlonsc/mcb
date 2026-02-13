use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Module for project type utility helpers.
pub mod project_type;
/// Module for submodule path utility helpers.
pub mod submodule;
pub mod vcs_context;

/// SHA-256 hex digest of content for deduplication.
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Mask sensitive ID for logging
///
/// Shows first 8 chars + "..." to allow correlation while protecting full ID.
/// Example: "ses_abc123def456" -> "ses_abc1..."
pub fn mask_id(id: &str) -> String {
    if id.len() <= 8 {
        id.to_string()
    } else {
        format!("{}...", &id[..8])
    }
}

/// Compute deterministic id hash for safe correlation.
///
/// Uses HMAC-SHA256 with `MCB_ID_HASH_SECRET` when configured.
/// Falls back to SHA-256 over `kind:raw_id` when secret is missing.
pub fn compute_stable_id_hash(kind: &str, raw_id: &str) -> String {
    let data = format!("{kind}:{raw_id}");

    if let Ok(secret) = std::env::var("MCB_ID_HASH_SECRET")
        && !secret.is_empty()
    {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC supports any key length");
        mac.update(data.as_bytes());
        return hex::encode(mac.finalize().into_bytes());
    }

    compute_content_hash(&data)
}
