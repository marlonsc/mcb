//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mcb_domain::error::Error;
use mcb_domain::ports::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
};
use mcb_domain::utils::analysis::{
    FunctionRecord, compute_complexity_score, count_symbol_occurrences, is_exempt_symbol,
};
use regex::Regex;
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

    /// Collects function definitions from files.
    fn collect_functions(files: &[(PathBuf, String)]) -> Result<Vec<FunctionRecord>> {
        let fn_re = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
            .map_err(|e| Error::invalid_argument(format!("invalid function regex: {e}")))?;

        let mut records = Vec::new();
        // TODO(qlty): Found 17 lines of identical code in 2 locations (mass = 105)
        for (file, content) in files {
            for captures in fn_re.captures_iter(content) {
                let Some(name_match) = captures.get(1) else {
                    continue;
                };
                let name = name_match.as_str().to_owned();
                let fn_start = name_match.start();
                let line = content[..fn_start].bytes().filter(|b| *b == b'\n').count() + 1;
                let complexity = compute_complexity_score(content, fn_start)?;
                records.push(FunctionRecord {
                    file: file.clone(),
                    name,
                    line,
                    complexity,
                });
            }
        }
        Ok(records)
    }

    // No longer needed: all_symbol_occurrences
}

/// Implementation of `ComplexityAnalyzer`.
impl ComplexityAnalyzer for NativePmatAnalyzer {
    // TODO(qlty): Found 20 lines of similar code in 2 locations (mass = 92)
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<ComplexityFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = Self::collect_functions(&files)?;

        Ok(functions
            .into_iter()
            .filter(|f| f.complexity > threshold)
            .map(|f| ComplexityFinding {
                file: f.file,
                function: f.name,
                complexity: f.complexity,
            })
            .collect())
    }
}

/// Native dead code detection using symbol counting.
impl DeadCodeDetector for NativePmatAnalyzer {
    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<DeadCodeFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = Self::collect_functions(&files)?;
        let names: Vec<String> = functions.iter().map(|f| f.name.clone()).collect();
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        let occurrences = count_symbol_occurrences(&contents, &names)?;

        let findings = functions
            .into_iter()
            .filter(|f| !is_exempt_symbol(&f.name))
            .filter(|f| occurrences.get(&f.name).copied().unwrap_or(0) <= 1)
            .map(|f| DeadCodeFinding {
                file: f.file,
                line: f.line,
                item_type: "function".to_owned(),
                name: f.name,
            })
            .collect();

        Ok(findings)
    }
}

/// Native technical debt scoring based on complexity and dead code.
impl TdgScorer for NativePmatAnalyzer {
    // TODO(qlty): Found 38 lines of identical code in 2 locations (mass = 266)
    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<TdgFinding>> {
        let files = Self::load_rust_files(workspace_root)?;
        let functions = Self::collect_functions(&files)?;

        let mut complexity_by_file: HashMap<PathBuf, u32> = HashMap::new();
        for function in functions {
            let entry = complexity_by_file.entry(function.file).or_insert(0);
            *entry = (*entry).max(function.complexity);
        }

        let dead_code = self.detect_dead_code(workspace_root)?;
        let mut dead_by_file: HashMap<PathBuf, u32> = HashMap::new();
        for finding in dead_code {
            *dead_by_file.entry(finding.file).or_insert(0) += 1;
        }

        let mut findings = Vec::new();
        for (path, content) in files {
            let sloc = content.lines().filter(|l| !l.trim().is_empty()).count() as u32;
            let complexity = complexity_by_file.get(&path).copied().unwrap_or(1);
            let dead = dead_by_file.get(&path).copied().unwrap_or(0);

            let tdg_score = complexity.saturating_mul(2)
                + dead.saturating_mul(10)
                + ((sloc / 200).saturating_mul(10));

            if tdg_score > threshold {
                findings.push(TdgFinding {
                    file: path,
                    score: tdg_score,
                });
            }
        }

        Ok(findings)
    }
}
