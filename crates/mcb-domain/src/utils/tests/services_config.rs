//!
//! **Documentation**: [docs/modules/domain.md](../../../docs/modules/domain.md)
//!
use std::path::PathBuf;
use std::sync::OnceLock;

/// Locates `config/tests.toml` by walking up from manifest and current directories.
#[must_use]
pub fn find_test_config_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        candidates.push(dir.join("config").join("tests.toml"));
    }

    if let Ok(current_dir) = std::env::current_dir() {
        for dir in current_dir.ancestors() {
            candidates.push(dir.join("config").join("tests.toml"));
        }
    }

    candidates.into_iter().find(|path| path.is_file())
}

/// Returns the cached `[test_services]` table from `config/tests.toml`.
///
/// Returns `None` if the config file is missing, unreadable, or malformed.
pub fn test_services_table() -> Option<&'static toml::value::Table> {
    static TEST_SERVICES: OnceLock<Option<toml::value::Table>> = OnceLock::new();

    TEST_SERVICES
        .get_or_init(|| {
            let config_path = find_test_config_path()?;
            let content = std::fs::read_to_string(&config_path).ok()?;
            let value = toml::from_str::<toml::Value>(&content).ok()?;
            value.get("test_services")?.as_table().cloned()
        })
        .as_ref()
}

/// Returns an optional service URL from `[test_services]`.
pub fn test_service_url(key: &str) -> Option<String> {
    test_services_table()?
        .get(key)
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
}

/// Returns a required service URL from `[test_services]`.
///
/// # Errors
///
/// Returns an error if the key is not found in `config/tests.toml` under `[test_services]`.
pub fn required_test_service_url(key: &str) -> Result<String, String> {
    test_service_url(key).ok_or_else(|| format!("missing test_services.{key} in config/tests.toml"))
}
