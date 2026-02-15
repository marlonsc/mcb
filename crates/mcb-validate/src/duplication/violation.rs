//! `DuplicationViolation` implementation.

use std::path::PathBuf;

use super::{CloneCandidate, DuplicationType};
use crate::traits::violation::{Severity, Violation, ViolationCategory};
use derive_more::Display;

/// A duplication violation representing a detected code clone
#[derive(Debug, Clone, Display)]
#[display(
    "{} at {}:{}: {} lines duplicated from {}:{}",
    duplication_type.name(),
    file.display(),
    line,
    duplicated_lines,
    duplicate_file.display(),
    duplicate_line
)]
pub struct DuplicationViolation {
    /// File containing the original code
    pub file: PathBuf,
    /// Line number of the original code (1-based)
    pub line: usize,
    /// File containing the duplicate
    pub duplicate_file: PathBuf,
    /// Line number of the duplicate (1-based)
    pub duplicate_line: usize,
    /// Type of duplication
    pub duplication_type: DuplicationType,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f64,
    /// Number of duplicated lines
    pub duplicated_lines: usize,
    /// Severity of the violation
    pub severity: Severity,
}

impl DuplicationViolation {
    /// Create a new duplication violation from a clone candidate
    pub fn from_candidate(candidate: &CloneCandidate) -> Self {
        let severity = match candidate.clone_type {
            DuplicationType::ExactClone | DuplicationType::RenamedClone => Severity::Warning,
            DuplicationType::GappedClone | DuplicationType::SemanticClone => Severity::Info,
        };
        Self {
            file: candidate.file1.clone(),
            line: candidate.start_line1,
            duplicate_file: candidate.file2.clone(),
            duplicate_line: candidate.start_line2,
            duplication_type: candidate.clone_type,
            similarity: candidate.similarity,
            duplicated_lines: candidate.duplicated_lines,
            severity,
        }
    }
}

impl Violation for DuplicationViolation {
    fn id(&self) -> &str {
        self.duplication_type.rule_id()
    }

    fn message(&self) -> String {
        format!(
            "{} detected: {} lines duplicated from {}:{}",
            self.duplication_type.name(),
            self.duplicated_lines,
            self.duplicate_file.display(),
            self.duplicate_line
        )
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&PathBuf> {
        Some(&self.file)
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn suggestion(&self) -> Option<String> {
        match self.duplication_type {
            DuplicationType::ExactClone => Some(
                "Extract the duplicated code into a shared function or module".to_string(),
            ),
            DuplicationType::RenamedClone => Some(
                "The code structure is identical with only renamed identifiers. Consider extracting with generics or parameters".to_string(),
            ),
            DuplicationType::GappedClone => Some(
                "Near-duplicate code detected. Consider refactoring into a common abstraction with small differences parameterized".to_string(),
            ),
            DuplicationType::SemanticClone => Some(
                "Functionally similar code detected. Review if a common interface or trait could reduce duplication".to_string(),
            ),
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }
}
