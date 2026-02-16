/// Extracts a canonical file path from tuple/path container variants.
///
/// # Example
///
/// ```rust
/// use mcb_validate::macros::ExtractFilePath;
/// let p = std::path::PathBuf::from("src/lib.rs");
/// assert_eq!(p.extract_file_path(), &p);
/// ```
pub trait ExtractFilePath {
    /// Returns the file-path component to use in violation location reporting.
    fn extract_file_path(&self) -> &std::path::PathBuf;
}

impl ExtractFilePath for std::path::PathBuf {
    fn extract_file_path(&self) -> &std::path::PathBuf {
        self
    }
}

impl<T> ExtractFilePath for (std::path::PathBuf, T) {
    fn extract_file_path(&self) -> &std::path::PathBuf {
        &self.0
    }
}

impl<T, U> ExtractFilePath for (std::path::PathBuf, T, U) {
    fn extract_file_path(&self) -> &std::path::PathBuf {
        &self.0
    }
}
