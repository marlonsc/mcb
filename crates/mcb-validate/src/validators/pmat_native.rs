//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rust_code_analysis::SpaceKind;

use mcb_domain::ports::{AnalysisFinding, CODE_ANALYZERS, CodeAnalyzer, CodeAnalyzerEntry};
use mcb_domain::utils::analysis::{self, FunctionRecord};

use crate::ast::rca_helpers;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, ValidationConfig};

#[derive(Debug, Default, Clone, Copy)]
struct NativePmatAnalyzer;

impl NativePmatAnalyzer {
    fn load_rust_files(workspace_root: &Path) -> Result<Vec<(PathBuf, String)>> {
        let config = ValidationConfig::new(workspace_root.to_path_buf());
        let mut files = Vec::new();
        for_each_scan_file(&config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            let content = fs::read_to_string(&entry.absolute_path)?;
            files.push((entry.absolute_path.clone(), content));
            Ok(())
        })?;
        mcb_domain::debug!(
            "pmat",
            "Loaded Rust files",
            &format!("count={}", files.len())
        );
        Ok(files)
    }

    /// Collect function records using RCA's AST and cyclomatic complexity metric,
    /// replacing the regex-based `analysis::collect_functions`.
    fn collect_functions_rca(files: &[(PathBuf, String)]) -> Vec<FunctionRecord> {
        let mut records = Vec::new();
        for (file, content) in files {
            let Some(root) = rca_helpers::parse_file_spaces(file, content) else {
                continue;
            };
            mcb_domain::trace!(
                "pmat",
                "Collecting functions",
                &format!("file={}", file.display())
            );
            for space in rca_helpers::collect_spaces_of_kind(&root, content, SpaceKind::Function) {
                let name = space.name.as_deref().unwrap_or("").to_owned();
                if name.is_empty() {
                    continue;
                }
                records.push(FunctionRecord {
                    file: file.clone(),
                    name,
                    line: space.start_line,
                    complexity: space.metrics.cyclomatic.cyclomatic().round() as u32,
                });
            }
        }
        mcb_domain::debug!(
            "pmat",
            "Functions collected",
            &format!("count={}", records.len())
        );
        records
    }
}

impl CodeAnalyzer for NativePmatAnalyzer {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> std::result::Result<Vec<AnalysisFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = Self::collect_functions_rca(&files);
        Ok(analysis::filter_complex_functions(functions, threshold))
    }

    fn detect_dead_code(
        &self,
        workspace_root: &Path,
    ) -> std::result::Result<Vec<AnalysisFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = Self::collect_functions_rca(&files);
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        analysis::detect_dead_functions(functions, &contents)
    }

    fn score_tdg(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> std::result::Result<Vec<AnalysisFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = Self::collect_functions_rca(&files);
        let dead_code = self.detect_dead_code(workspace_root)?;
        Ok(analysis::compute_tdg_scores(
            &files, functions, &dead_code, threshold,
        ))
    }
}

// Auto-registration via linkme distributed slice
#[allow(unsafe_code)]
#[linkme::distributed_slice(CODE_ANALYZERS)]
static NATIVE_RCA_ANALYZER: CodeAnalyzerEntry = CodeAnalyzerEntry {
    name: "native-rca",
    description: "RCA/tree-sitter code analyzer",
    build: || Ok(Arc::new(NativePmatAnalyzer)),
};
