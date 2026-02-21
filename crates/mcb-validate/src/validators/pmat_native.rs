//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
use std::fs;
use std::path::{Path, PathBuf};

use mcb_domain::ports::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
};
use mcb_domain::utils::analysis;

use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, ValidationConfig};

#[derive(Debug, Default, Clone, Copy)]
pub struct NativePmatAnalyzer;

impl NativePmatAnalyzer {
    fn load_rust_files(workspace_root: &Path) -> Result<Vec<(PathBuf, String)>> {
        let config = ValidationConfig::new(workspace_root.to_path_buf());
        let mut files = Vec::new();
        for_each_scan_file(&config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            let content = fs::read_to_string(&entry.absolute_path)?;
            files.push((entry.absolute_path.clone(), content));
            Ok(())
        })?;
        Ok(files)
    }
}

impl ComplexityAnalyzer for NativePmatAnalyzer {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> std::result::Result<Vec<ComplexityFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = analysis::collect_functions(&files)?;
        Ok(analysis::filter_complex_functions(functions, threshold))
    }
}

impl DeadCodeDetector for NativePmatAnalyzer {
    fn detect_dead_code(
        &self,
        workspace_root: &Path,
    ) -> std::result::Result<Vec<DeadCodeFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = analysis::collect_functions(&files)?;
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        analysis::detect_dead_functions(functions, &contents)
    }
}

impl TdgScorer for NativePmatAnalyzer {
    fn score_tdg(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> std::result::Result<Vec<TdgFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = analysis::collect_functions(&files)?;
        let dead_code = self.detect_dead_code(workspace_root)?;
        Ok(analysis::compute_tdg_scores(
            &files, functions, &dead_code, threshold,
        ))
    }
}
