//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_domain::value_objects::CollectionId;

/// Validate and normalize a user-supplied collection name into a valid [`CollectionId`].
///
/// Returns an error string when the name is empty or contains characters outside
/// the allowed set (`[a-zA-Z0-9_\-.]`).  Hyphens and dots are replaced with
/// underscores during normalization so the resulting identifier is safe for
/// vector-store backends that only accept `[a-z0-9_]`.
///
/// # Errors
/// Returns an error when the input is empty, too long, or contains unsupported characters.
pub fn normalize_collection_name(user_name: &str) -> Result<CollectionId, String> {
    if user_name.is_empty() {
        return Err("collection name cannot be empty".into());
    }
    if user_name.len() > 255 {
        return Err("collection name exceeds maximum length (255)".into());
    }
    if !user_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err("collection name contains invalid characters".into());
    }
    let normalized = user_name.replace(['-', '.'], "_").to_lowercase();
    Ok(CollectionId::from_name(&normalized))
}
