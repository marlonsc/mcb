use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mcb_domain::error::Error;
use mcb_domain::ports::providers::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
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
                && !path.to_string_lossy().contains("/target/")
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

    /// Counts occurrences of symbols across files.
    fn all_symbol_occurrences(
        files: &[(PathBuf, String)],
        symbols: &[String],
    ) -> Result<HashMap<String, usize>> {
        let mut map = HashMap::new();
        for symbol in symbols {
            let escaped = regex::escape(symbol);
            let re = Regex::new(&format!(r"\b{escaped}\b"))
                .map_err(|e| Error::invalid_argument(format!("invalid symbol regex: {e}")))?;
            let count = files
                .iter()
                .map(|(_, content)| re.find_iter(content).count())
                .sum();
            map.insert(symbol.clone(), count);
        }
        Ok(map)
    }
}

/// Implementation of ComplexityAnalyzer.
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
        let occurrences = Self::all_symbol_occurrences(&files, &names)?;

        let findings = functions
            .into_iter()
            .filter(|f| !is_exempt_symbol(&f.name))
            .filter(|f| occurrences.get(&f.name).copied().unwrap_or(0) <= 1)
            .map(|f| DeadCodeFinding {
                file: f.file,
                line: f.line,
                item_type: "function".to_string(),
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

/// Record of a discovered function.
#[derive(Debug, Clone)]
struct FunctionRecord {
    file: PathBuf,
    name: String,
    line: usize,
    complexity: u32,
}

/// Computes cyclomatic complexity using control flow keyword counting.
fn compute_complexity_score(content: &str, start_pos: usize) -> Result<u32> {
    let body = extract_function_body(content, start_pos).unwrap_or_default();
    let re = Regex::new(r"\b(if|for|while|loop|match)\b|&&|\|\|")
        .map_err(|e| Error::invalid_argument(format!("invalid complexity regex: {e}")))?;
    let count = re.find_iter(&body).count() as u32;
    Ok(1 + count)
}

/// Extracts the function body by balancing braces.
// TODO(qlty): Found 23 lines of similar code in 2 locations (mass = 109)
fn extract_function_body(content: &str, start_pos: usize) -> Option<String> {
    let after_start = &content[start_pos..];
    let brace_index = after_start.find('{')?;
    let body_start = start_pos + brace_index;

    let bytes = content.as_bytes();
    let mut depth = 0_i32;
    let mut i = body_start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(content[body_start..=i].to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Checks if a symbol should be exempt from dead code detection (e.g. main, tests).
fn is_exempt_symbol(name: &str) -> bool {
    matches!(name, "main") || name.starts_with("test_")
}
