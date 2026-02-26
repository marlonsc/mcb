use std::path::Path;

pub(crate) async fn read_file_opt(path: &Path, detector: &str) -> Option<String> {
    match tokio::fs::read_to_string(path).await {
        Ok(content) => Some(content),
        Err(e) => {
            mcb_domain::debug!(
                detector,
                "Failed to read file",
                &format!("path={path:?}, err={e}")
            );
            None
        }
    }
}

pub(crate) fn parse_json_opt<T: serde::de::DeserializeOwned>(
    content: &str,
    path: &Path,
    detector: &str,
) -> Option<T> {
    match serde_json::from_str(content) {
        Ok(v) => Some(v),
        Err(e) => {
            mcb_domain::debug!(
                detector,
                "Failed to parse JSON",
                &format!("path={path:?}, err={e}")
            );
            None
        }
    }
}

pub(crate) fn parse_toml_opt<T: serde::de::DeserializeOwned>(
    content: &str,
    path: &Path,
    detector: &str,
) -> Option<T> {
    match toml::from_str(content) {
        Ok(v) => Some(v),
        Err(e) => {
            mcb_domain::debug!(
                detector,
                "Failed to parse TOML",
                &format!("path={path:?}, err={e}")
            );
            None
        }
    }
}
