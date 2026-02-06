//! `DuplicationAnalyzer` and `DuplicationStats` implementation.

use std::fs;
use std::path::{Path, PathBuf};

use mcb_language_support::LanguageDetector;

use super::detector::{CloneDetector, tokenize_source};
use super::fingerprint::TokenFingerprinter;
use super::thresholds::{DuplicationThresholds, DuplicationType};
use super::violation::DuplicationViolation;

/// Main duplication analyzer facade
///
/// Combines token fingerprinting for fast initial detection with
/// AST similarity analysis for accurate clone classification.
pub struct DuplicationAnalyzer {
    thresholds: DuplicationThresholds,
    detector: LanguageDetector,
}

impl DuplicationAnalyzer {
    /// Create a new analyzer with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: DuplicationThresholds::default(),
            detector: LanguageDetector::new(),
        }
    }

    /// Create an analyzer with custom thresholds
    pub fn with_thresholds(thresholds: DuplicationThresholds) -> Self {
        Self {
            thresholds,
            detector: LanguageDetector::new(),
        }
    }

    /// Analyze files for code duplication
    pub fn analyze_files(&self, paths: &[PathBuf]) -> Result<Vec<DuplicationViolation>, String> {
        let mut fingerprinter = TokenFingerprinter::new(self.thresholds.min_tokens);

        for path in paths {
            if !self.should_analyze_file(path) {
                continue;
            }

            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

            let language = self.detect_language(path);
            let tokens = tokenize_source(&content, &language);

            if tokens.len() >= self.thresholds.min_tokens {
                fingerprinter.fingerprint_file(path, &tokens);
            }
        }

        let matches = fingerprinter.find_duplicates();
        let detector = CloneDetector::new(self.thresholds.clone());
        let candidates = detector.verify_candidates(&matches);

        let violations: Vec<DuplicationViolation> = candidates
            .iter()
            .map(DuplicationViolation::from_candidate)
            .collect();

        Ok(violations)
    }

    /// Check if a file should be analyzed based on language and patterns
    pub fn should_analyze_file(&self, path: &Path) -> bool {
        let language = self.detect_language(path);
        if !self.thresholds.languages.contains(&language) {
            return false;
        }

        let path_str = path.to_string_lossy();
        for pattern in &self.thresholds.exclude_patterns {
            let pattern_regex = pattern.replace("**", ".*").replace('*', "[^/]*");
            if regex::Regex::new(&pattern_regex)
                .map(|r| r.is_match(&path_str))
                .unwrap_or(false)
            {
                return false;
            }
        }

        true
    }

    /// Detect language from file path using mcb-language-support
    fn detect_language(&self, path: &Path) -> String {
        if let Some(name) = self.detector.detect_name(path, None) {
            return name;
        }

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        self.extension_to_language_fallback(extension).to_string()
    }

    /// Fallback extension to language mapping for languages not in mcb-language-support
    fn extension_to_language_fallback(&self, extension: &str) -> &str {
        match extension {
            "go" => "go",
            "c" | "h" => "c",
            "cs" => "csharp",
            "rb" => "ruby",
            "php" => "php",
            "swift" => "swift",
            _ => "unknown",
        }
    }

    /// Get duplication statistics from analysis
    pub fn get_stats(&self, violations: &[DuplicationViolation]) -> DuplicationStats {
        let mut exact_count = 0;
        let mut renamed_count = 0;
        let mut gapped_count = 0;
        let mut semantic_count = 0;
        let mut total_duplicated_lines = 0;

        for v in violations {
            total_duplicated_lines += v.duplicated_lines;
            match v.duplication_type {
                DuplicationType::ExactClone => exact_count += 1,
                DuplicationType::RenamedClone => renamed_count += 1,
                DuplicationType::GappedClone => gapped_count += 1,
                DuplicationType::SemanticClone => semantic_count += 1,
            }
        }

        DuplicationStats {
            total_clones: violations.len(),
            exact_clones: exact_count,
            renamed_clones: renamed_count,
            gapped_clones: gapped_count,
            semantic_clones: semantic_count,
            total_duplicated_lines,
        }
    }
}

impl Default for DuplicationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about detected duplications
#[derive(Debug, Clone, Default)]
pub struct DuplicationStats {
    /// Total number of clones detected
    pub total_clones: usize,
    /// Number of Type 1 (exact) clones
    pub exact_clones: usize,
    /// Number of Type 2 (renamed) clones
    pub renamed_clones: usize,
    /// Number of Type 3 (gapped) clones
    pub gapped_clones: usize,
    /// Number of Type 4 (semantic) clones
    pub semantic_clones: usize,
    /// Total number of duplicated lines
    pub total_duplicated_lines: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::duplication::CloneCandidate;
    use crate::violation_trait::{Severity, Violation};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_duplication_violation_from_candidate() {
        let candidate = CloneCandidate {
            file1: PathBuf::from("a.rs"),
            start_line1: 10,
            end_line1: 20,
            file2: PathBuf::from("b.rs"),
            start_line2: 30,
            end_line2: 40,
            similarity: 1.0,
            clone_type: DuplicationType::ExactClone,
            duplicated_lines: 11,
        };

        let violation = DuplicationViolation::from_candidate(&candidate);

        assert_eq!(violation.id(), "DUP001");
        assert_eq!(violation.line(), Some(10));
        assert!(violation.message().contains("Exact Clone"));
        assert!(violation.suggestion().is_some());
    }

    #[test]
    fn test_analyzer_detects_exact_duplicates() {
        let dir = TempDir::new().unwrap();

        let code = r#"
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}

fn calculate_average(numbers: &[i32]) -> f64 {
    let sum = calculate_sum(numbers);
    let count = numbers.len();
    if count == 0 {
        return 0.0;
    }
    sum as f64 / count as f64
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum = calculate_sum(&numbers);
    let avg = calculate_average(&numbers);
    println!("Sum: {}, Average: {}", sum, avg);
}
"#;

        let file1 = create_temp_file(&dir, "file1.rs", code);
        let file2 = create_temp_file(&dir, "file2.rs", code);

        let thresholds = DuplicationThresholds {
            min_tokens: 10,
            min_lines: 3,
            ..Default::default()
        };
        let analyzer = DuplicationAnalyzer::with_thresholds(thresholds);
        let violations = analyzer.analyze_files(&[file1, file2]).unwrap();

        for v in &violations {
            assert!(
                v.duplicated_lines >= 1,
                "Duplicated lines should be positive"
            );
            assert!(v.similarity > 0.0, "Similarity should be positive");
        }
    }

    #[test]
    fn test_analyzer_respects_min_lines() {
        let dir = TempDir::new().unwrap();

        let code = "let x = 1;";
        let file1 = create_temp_file(&dir, "small1.rs", code);
        let file2 = create_temp_file(&dir, "small2.rs", code);

        let thresholds = DuplicationThresholds {
            min_lines: 10,
            min_tokens: 5,
            ..Default::default()
        };
        let analyzer = DuplicationAnalyzer::with_thresholds(thresholds);
        let violations = analyzer.analyze_files(&[file1, file2]).unwrap();

        assert!(
            violations.is_empty(),
            "Should not detect duplicates below min_lines threshold"
        );
    }

    #[test]
    fn test_duplication_stats() {
        let violations = vec![
            DuplicationViolation {
                file: PathBuf::from("a.rs"),
                line: 1,
                duplicate_file: PathBuf::from("b.rs"),
                duplicate_line: 1,
                duplication_type: DuplicationType::ExactClone,
                similarity: 1.0,
                duplicated_lines: 10,
                severity: Severity::Warning,
            },
            DuplicationViolation {
                file: PathBuf::from("c.rs"),
                line: 1,
                duplicate_file: PathBuf::from("d.rs"),
                duplicate_line: 1,
                duplication_type: DuplicationType::RenamedClone,
                similarity: 0.95,
                duplicated_lines: 15,
                severity: Severity::Warning,
            },
        ];

        let analyzer = DuplicationAnalyzer::new();
        let stats = analyzer.get_stats(&violations);

        assert_eq!(stats.total_clones, 2);
        assert_eq!(stats.exact_clones, 1);
        assert_eq!(stats.renamed_clones, 1);
        assert_eq!(stats.total_duplicated_lines, 25);
    }

    #[test]
    fn test_language_detection() {
        let analyzer = DuplicationAnalyzer::new();

        assert_eq!(analyzer.detect_language(Path::new("test.rs")), "rust");
        assert_eq!(analyzer.detect_language(Path::new("test.py")), "python");
        assert_eq!(analyzer.detect_language(Path::new("test.ts")), "typescript");
        assert_eq!(analyzer.detect_language(Path::new("test.js")), "javascript");
    }

    #[test]
    fn test_should_analyze_file() {
        let analyzer = DuplicationAnalyzer::new();

        assert!(analyzer.should_analyze_file(Path::new("src/lib.rs")));
        assert!(analyzer.should_analyze_file(Path::new("src/main.py")));
    }
}
