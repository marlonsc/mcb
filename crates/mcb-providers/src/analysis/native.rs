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

    fn collect_functions(files: &[(PathBuf, String)]) -> Result<Vec<FunctionRecord>> {
        let fn_re = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
            .map_err(|e| Error::invalid_argument(format!("invalid function regex: {e}")))?;

        let mut records = Vec::new();
        for (file, content) in files {
            for captures in fn_re.captures_iter(content) {
                let Some(name_match) = captures.get(1) else {
                    continue;
                };
                let name = name_match.as_str().to_string();
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

impl ComplexityAnalyzer for NativePmatAnalyzer {
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

impl TdgScorer for NativePmatAnalyzer {
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

#[derive(Debug, Clone)]
struct FunctionRecord {
    file: PathBuf,
    name: String,
    line: usize,
    complexity: u32,
}

fn compute_complexity_score(content: &str, start_pos: usize) -> Result<u32> {
    let body = extract_function_body(content, start_pos).unwrap_or_default();
    let re = Regex::new(r"\b(if|for|while|loop|match)\b|&&|\|\|")
        .map_err(|e| Error::invalid_argument(format!("invalid complexity regex: {e}")))?;
    let count = re.find_iter(&body).count() as u32;
    Ok(1 + count)
}

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

fn is_exempt_symbol(name: &str) -> bool {
    matches!(name, "main") || name.starts_with("test_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_high_complexity_functions() {
        let temp = TempDir::new().expect("create tempdir");
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            r#"
fn simple() { let x = 1; }

fn complex(a: i32) {
    if a > 0 { }
    for _i in 0..10 { }
    while a > 1 { break; }
    match a { 1 => (), _ => () }
}
"#,
        )
        .expect("write sample");

        let analyzer = NativePmatAnalyzer;
        let findings = analyzer
            .analyze_complexity(temp.path(), 3)
            .expect("analyze complexity");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].function, "complex");
    }

    #[test]
    fn detects_dead_code_functions() {
        let temp = TempDir::new().expect("create tempdir");
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            r#"
fn used() {}
fn dead_fn() {}

fn caller() {
    used();
}
"#,
        )
        .expect("write sample");

        let analyzer = NativePmatAnalyzer;
        let findings = analyzer
            .detect_dead_code(temp.path())
            .expect("detect dead code");

        assert!(findings.iter().any(|f| f.name == "dead_fn"));
    }

    #[test]
    fn computes_tdg_score_above_threshold() {
        let temp = TempDir::new().expect("create tempdir");
        let file = temp.path().join("sample.rs");
        fs::write(
            &file,
            r#"
fn dead_a() {}
fn dead_b() {}
fn heavy(x: i32) {
    if x > 0 {}
    if x > 1 {}
    if x > 2 {}
    if x > 3 {}
}
"#,
        )
        .expect("write sample");

        let analyzer = NativePmatAnalyzer;
        let findings = analyzer.score_tdg(temp.path(), 15).expect("score tdg");

        assert_eq!(findings.len(), 1);
        assert!(findings[0].score > 15);
    }
}
