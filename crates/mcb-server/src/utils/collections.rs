use mcb_domain::error::Result;
use mcb_domain::value_objects::CollectionId;

pub fn normalize_collection_name(user_name: &str) -> Result<CollectionId> {
    let normalized = user_name.replace('-', "_").to_lowercase();
    Ok(CollectionId::new(normalized))
}
