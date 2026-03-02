//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_domain::error::Error;
use mcb_domain::error::Result;
use mcb_domain::ports::{AnalysisFinding, CODE_ANALYZERS, CodeAnalyzer, CodeAnalyzerEntry};
use mcb_domain::utils::analysis;
use walkdir::WalkDir;

/// Native PMAT-style analyzer implementation.
#[derive(Debug, Default, Clone, Copy)]
struct NativePmatAnalyzer;

impl NativePmatAnalyzer {
    /// Loads Rust files from the workspace.
    fn load_rust_files(workspace_root: &Path) -> Result<Vec<(PathBuf, String)>> {
        let mut files = Vec::new();
        for entry in WalkDir::new(workspace_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "rs")
                && !path.to_str().is_some_and(|s| s.contains("/target/"))
            {
                let content = fs::read_to_string(path).map_err(|e| {
                    Error::io_with_source(format!("failed to read {}", path.display()), e)
                })?;
                files.push((path.to_path_buf(), content));
            }
        }
        Ok(files)
    }
}

impl CodeAnalyzer for NativePmatAnalyzer {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<AnalysisFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        Ok(analysis::filter_complex_functions(functions, threshold))
    }

    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<AnalysisFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        analysis::detect_dead_functions(functions, &contents)
    }

    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<AnalysisFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        let dead_code = self.detect_dead_code(workspace_root)?;
        Ok(analysis::compute_tdg_scores(
            &files, functions, &dead_code, threshold,
        ))
    }
}

// Auto-registration via linkme distributed slice
#[allow(unsafe_code)]
#[linkme::distributed_slice(CODE_ANALYZERS)]
static NATIVE_REGEX_ANALYZER: CodeAnalyzerEntry = CodeAnalyzerEntry {
    name: "native-regex",
    description: "Regex-based code analyzer",
    build: || Ok(Arc::new(NativePmatAnalyzer)),
};
