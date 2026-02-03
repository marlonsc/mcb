use crate::entities::submodule::SubmoduleInfo;

/// Helpers for working with submodules in Phase 6 hybrid search.
pub fn collection_name(info: &SubmoduleInfo, parent_collection: &str) -> String {
    format!("{}/{}", parent_collection, info.path.replace('/', "-"))
}

/// Stable repo identifier for a submodule, used when populating hybrid search metadata.
pub fn repo_id(info: &SubmoduleInfo) -> String {
    format!("{}:{}", info.parent_repo_id, info.path)
}
