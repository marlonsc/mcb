//! Collection Name Mapping Manager
//!
//! Manages the mapping between user-friendly collection names (with hyphens)
//! and Milvus-compatible names (with underscores and timestamp suffix).
//!
//! Stores mapping in `~/.config/mcb/collection_mapping.json`
//!
//! Uses file locking (flock) to prevent corruption from concurrent access.
//!
//! Example:
//! ```json
//! {
//!   "mcb": "mcp_context_browser_20260126_143021",
//!   "my-project": "my_project_20260126_143022"
//! }
//! ```

// use crate::constants::{COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::config::{
    COLLECTION_MAPPING_FILENAME, COLLECTION_MAPPING_LOCK_FILENAME, config_dir,
};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

/// Gets the default mapping file path (~/.config/mcb/collection_mapping.json)
fn get_mapping_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(COLLECTION_MAPPING_FILENAME))
}

/// Gets the lock file path
fn get_lock_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(COLLECTION_MAPPING_LOCK_FILENAME))
}

/// RAII guard for file locking
struct FileLockGuard {
    _file: File,
}

use fs2::FileExt;

impl FileLockGuard {
    /// Acquire an exclusive lock on the mapping file
    fn acquire() -> Result<Self> {
        let lock_path = get_lock_file_path()?;

        // Ensure directory exists
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("Failed to create config directory: {}", e)))?;
        }

        // Open/create the lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| Error::io(format!("Failed to open lock file: {}", e)))?;

        // Acquire exclusive lock (blocking)
        file.lock_exclusive()
            .map_err(|e| Error::io(format!("Failed to acquire file lock: {}", e)))?;

        Ok(Self { _file: file })
    }
}

/// Load the collection name mapping from disk (internal, no locking)
fn load_mapping_internal(mapping_path: &PathBuf) -> Result<HashMap<String, String>> {
    if !mapping_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(mapping_path)
        .map_err(|e| Error::io(format!("Failed to read mapping file: {}", e)))?;

    // Handle empty file
    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    serde_json::from_str(&content)
        .map_err(|e| Error::io(format!("Failed to parse mapping file: {}", e)))
}

/// Save the collection name mapping to disk using atomic write
fn save_mapping_atomic(mapping: &HashMap<String, String>, mapping_path: &PathBuf) -> Result<()> {
    // Ensure directory exists
    if let Some(parent) = mapping_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Failed to create config directory: {}", e)))?;
    }

    let json = serde_json::to_string_pretty(mapping)
        .map_err(|e| Error::io(format!("Failed to serialize mapping: {}", e)))?;

    // Write to temp file first, then rename (atomic on most filesystems)
    let temp_path = mapping_path.with_extension("json.tmp");

    std::fs::write(&temp_path, &json)
        .map_err(|e| Error::io(format!("Failed to write temp mapping file: {}", e)))?;

    std::fs::rename(&temp_path, mapping_path)
        .map_err(|e| Error::io(format!("Failed to rename mapping file: {}", e)))
}

/// Generate a Milvus-compatible name from a user-friendly collection name
fn generate_milvus_name(user_name: &str) -> String {
    // Replace hyphens with underscores
    let normalized = user_name.replace('-', "_").to_lowercase();

    // Add timestamp suffix to prevent collisions
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let timestamp = format!("{}", now.as_secs() % 1_000_000); // Last 6 digits

    format!("{}_{}", normalized, timestamp)
}

/// Get or create a Milvus-compatible collection name
///
/// Uses file locking to ensure thread-safe access to the mapping file.
///
/// # Arguments
/// * `user_name` - User-provided collection name (e.g., "mcb")
///
/// # Returns
/// * `CollectionId` - Milvus-compatible name (stored in mapping)
///
/// # Example
/// ```no_run
/// use mcb_server::collection_mapping::map_collection_name;
///
/// let milvus_name = map_collection_name("mcb").unwrap();
/// // Returns: CollectionId("mcp_context_browser_143021")
/// ```
pub fn map_collection_name(user_name: &str) -> Result<CollectionId> {
    let mapping_path = get_mapping_file_path()?;

    // Acquire exclusive lock for the entire read-modify-write operation
    let _lock = FileLockGuard::acquire()?;

    let mut mapping = load_mapping_internal(&mapping_path)?;

    // Return existing mapping if available
    if let Some(milvus_name) = mapping.get(user_name) {
        return Ok(CollectionId::new(milvus_name));
    }

    // Generate new mapping
    let milvus_name = generate_milvus_name(user_name);
    mapping.insert(user_name.to_string(), milvus_name.clone());

    // Persist the mapping atomically
    save_mapping_atomic(&mapping, &mapping_path)?;

    Ok(CollectionId::new(milvus_name))
    // Lock is released when _lock goes out of scope
}

/// Get all known collections (user-friendly names)
///
/// # Returns
/// * `Vec<String>` - List of user-provided collection names
pub fn list_collections() -> Result<Vec<String>> {
    let mapping_path = get_mapping_file_path()?;
    let _lock = FileLockGuard::acquire()?;
    let mapping = load_mapping_internal(&mapping_path)?;
    let mut collections: Vec<String> = mapping.keys().cloned().collect();
    collections.sort();
    Ok(collections)
}

/// Get the reverse mapping (Milvus name → user name)
///
/// # Returns
/// * `HashMap<String, String>` - Mapping from Milvus names to user names
pub fn get_reverse_mapping() -> Result<HashMap<String, String>> {
    let mapping_path = get_mapping_file_path()?;
    let _lock = FileLockGuard::acquire()?;
    let mapping = load_mapping_internal(&mapping_path)?;
    let reversed = mapping
        .into_iter()
        .map(|(user, milvus)| (milvus, user))
        .collect();
    Ok(reversed)
}
