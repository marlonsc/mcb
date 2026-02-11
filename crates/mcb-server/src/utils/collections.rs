use mcb_domain::error::Error;
use mcb_domain::error::Result;
use mcb_domain::value_objects::CollectionId;

/// Normalize a user-supplied collection name into a valid [`CollectionId`].
pub fn normalize_collection_name(user_name: &str) -> Result<CollectionId> {
    let trimmed = user_name.trim();
    if trimmed.is_empty() {
        return Err(Error::invalid_argument("collection name cannot be empty"));
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(Error::invalid_argument(
            "collection name must contain only ASCII letters, numbers, '-' or '_'",
        ));
    }

    let normalized = trimmed.replace('-', "_").to_lowercase();
    Ok(CollectionId::new(normalized))
}
