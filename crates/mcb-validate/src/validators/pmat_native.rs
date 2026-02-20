use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, ValidationConfig};
use mcb_domain::ports::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
};
use mcb_domain::utils::analysis::{
    FunctionRecord, compute_complexity_score, count_symbol_occurrences, is_exempt_symbol,
};

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

    fn collect_functions(files: &[(PathBuf, String)]) -> Result<Vec<FunctionRecord>> {
        let fn_re =
            compile_regex(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")?;

        let mut records = Vec::new();
        for (file, content) in files {
            for captures in fn_re.captures_iter(content) {
                let Some(name_match) = captures.get(1) else {
                    continue;
                };
                let name = name_match.as_str().to_owned();
                let fn_start = name_match.start();
                let line = content[..fn_start].bytes().filter(|b| *b == b'\n').count() + 1;
                // Use domain analysis util
                let complexity = compute_complexity_score(content, fn_start)
                    .map_err(|e| crate::ValidationError::Config(e.to_string()))?;
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
}

impl ComplexityAnalyzer for NativePmatAnalyzer {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> std::result::Result<Vec<ComplexityFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = Self::collect_functions(&files)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
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

impl DeadCodeDetector for NativePmatAnalyzer {
    fn detect_dead_code(
        &self,
        workspace_root: &Path,
    ) -> std::result::Result<Vec<DeadCodeFinding>, mcb_domain::error::Error> {
        let files = Self::load_rust_files(workspace_root)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let functions = Self::collect_functions(&files)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;
        let names: Vec<String> = functions.iter().map(|f| f.name.clone()).collect();
        let contents: Vec<String> = files.iter().map(|(_, c)| c.clone()).collect();
        // Use domain analysis util
        let occurrences = count_symbol_occurrences(&contents, &names)?;

        Ok(functions
            .into_iter()
            .filter(|f| !is_exempt_symbol(&f.name))
            .filter(|f| occurrences.get(&f.name).copied().unwrap_or(0) <= 1)
            .map(|f| DeadCodeFinding {
                file: f.file,
                line: f.line,
                item_type: "function".to_owned(),
                name: f.name,
            })
            .collect())
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
        let functions = Self::collect_functions(&files)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;

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
            let sloc = content
                .lines()
                .filter(|l: &&str| !l.trim().is_empty())
                .count() as u32;
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
