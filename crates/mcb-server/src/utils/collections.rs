use mcb_domain::value_objects::CollectionId;

/// Normalize a user-supplied collection name into a valid [`CollectionId`].
///
/// The input has all `-` characters replaced with `_` and is converted to lowercase
/// before being used to construct a `CollectionId`.
///
/// # Examples
///
/// ```
/// let id: CollectionId = normalize_collection_name("My-Collection");
/// ```
pub fn normalize_collection_name(user_name: &str) -> CollectionId {
    let normalized = user_name.replace('-', "_").to_lowercase();
    CollectionId::new(normalized)
}