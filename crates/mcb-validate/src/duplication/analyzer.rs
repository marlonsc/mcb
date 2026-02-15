//! `DuplicationAnalyzer` and `DuplicationStats` implementation.

use std::fs;
use std::path::{Path, PathBuf};

use crate::filters::LanguageDetector;

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

        let Some(path_str) = path.to_str() else {
            return false;
        };
        for pattern in &self.thresholds.exclude_patterns {
            let pattern_regex = pattern.replace("**", ".*").replace('*', "[^/]*");
            if regex::Regex::new(&pattern_regex)
                .map(|r| r.is_match(path_str))
                .unwrap_or(false)
            {
                return false;
            }
        }

        true
    }

    fn detect_language(&self, path: &Path) -> String {
        if let Some(name) = self.detector.detect_name(path, None) {
            return name;
        }

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        self.extension_to_language_fallback(extension).to_string()
    }

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
