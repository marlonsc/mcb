use std::path::PathBuf;
use std::sync::OnceLock;

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

pub fn test_services_table() -> Option<&'static toml::value::Table> {
    static TEST_SERVICES: OnceLock<Option<toml::value::Table>> = OnceLock::new();

    TEST_SERVICES
        .get_or_init(|| {
            let config_path = find_test_config_path().expect(
                "CRITICAL: config/tests.toml not found! Integration tests require this configuration.",
            );
            let content = std::fs::read_to_string(&config_path)
                .unwrap_or_else(|e| panic!("Failed to read config file at {:?}: {}", config_path, e));
            let value = toml::from_str::<toml::Value>(&content).unwrap_or_else(|e| {
                panic!("Failed to parse TOML from {:?}: {}", config_path, e)
            });
            
            match value.get("test_services") {
                Some(v) => Some(v.as_table().unwrap_or_else(|| {
                    panic!("'test_services' in {:?} must be a table", config_path)
                }).clone()),
                None => panic!("Missing [test_services] table in {:?}", config_path),
            }
        })
        .as_ref()
}

pub fn test_service_url(key: &str) -> Option<String> {
    test_services_table()?
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

pub fn required_test_service_url(key: &str) -> String {
    test_service_url(key)
        .unwrap_or_else(|| panic!("missing test_services.{key} in config/tests.toml"))
}
