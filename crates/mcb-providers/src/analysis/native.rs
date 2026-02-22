//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
use std::fs;
use std::path::{Path, PathBuf};

use mcb_domain::error::Error;
use mcb_domain::ports::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
};
use mcb_domain::utils::analysis;
use walkdir::WalkDir;

use crate::Result;

/// Native PMAT-style analyzer implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativePmatAnalyzer;

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

impl ComplexityAnalyzer for NativePmatAnalyzer {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<ComplexityFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        Ok(analysis::filter_complex_functions(functions, threshold))
    }
}

impl DeadCodeDetector for NativePmatAnalyzer {
    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<DeadCodeFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        analysis::detect_dead_functions(functions, &contents)
    }
}

impl TdgScorer for NativePmatAnalyzer {
    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<TdgFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = analysis::collect_functions(&files)?;
        let dead_code = self.detect_dead_code(workspace_root)?;
        Ok(analysis::compute_tdg_scores(
            &files, functions, &dead_code, threshold,
        ))
    }
}
