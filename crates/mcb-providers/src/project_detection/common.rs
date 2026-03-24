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

/// Parse content with the given parser, logging failures as debug.
macro_rules! parse_opt {
    ($content:expr, $path:expr, $format:literal, $detector:expr, $parser:path) => {
        match $parser($content) {
            Ok(v) => Some(v),
            Err(e) => {
                mcb_domain::debug!(
                    $detector,
                    concat!("Failed to parse ", $format),
                    &format!("path={:?}, err={e}", $path)
                );
                None
            }
        }
    };
}

pub(crate) fn parse_json_opt<T: serde::de::DeserializeOwned>(
    content: &str,
    path: &Path,
    detector: &str,
) -> Option<T> {
    parse_opt!(content, path, "JSON", detector, serde_json::from_str)
}

pub(crate) fn parse_toml_opt<T: serde::de::DeserializeOwned>(
    content: &str,
    path: &Path,
    detector: &str,
) -> Option<T> {
    parse_opt!(content, path, "TOML", detector, toml::from_str)
}
