//! Duplication Detection Thresholds and Types
//!
//! Defines duplication types (clone categories) and configurable thresholds
//! for the duplication detection system.

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Clone type classification following established taxonomy
///
/// - Type 1 (Exact): Identical code fragments
/// - Type 2 (Renamed): Code with renamed identifiers
/// - Type 3 (Gapped): Near-miss clones with small modifications
/// - Type 4 (Semantic): Functionally equivalent code (future)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum DuplicationType {
    /// Type 1: Exact copy-paste (100% identical)
    #[display("Exact Clone")]
    ExactClone,
    /// Type 2: Renamed identifiers only
    #[display("Renamed Clone")]
    RenamedClone,
    /// Type 3: Near-miss with small modifications
    #[display("Gapped Clone")]
    GappedClone,
    /// Type 4: Functionally similar (future implementation)
    #[display("Semantic Clone")]
    SemanticClone,
}

impl DuplicationType {
    /// Get the rule ID prefix for this duplication type
    #[must_use]
    pub fn rule_id(&self) -> &'static str {
        match self {
            DuplicationType::ExactClone => "DUP001",
            DuplicationType::RenamedClone => "DUP002",
            DuplicationType::GappedClone => "DUP003",
            DuplicationType::SemanticClone => "DUP004",
        }
    }

    /// Get human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            DuplicationType::ExactClone => "Exact Clone",
            DuplicationType::RenamedClone => "Renamed Clone",
            DuplicationType::GappedClone => "Gapped Clone",
            DuplicationType::SemanticClone => "Semantic Clone",
        }
    }

    /// Get minimum similarity threshold for this type
    #[must_use]
    pub fn min_similarity(&self) -> f64 {
        match self {
            DuplicationType::ExactClone => 1.0,
            DuplicationType::RenamedClone => 0.95,
            DuplicationType::GappedClone => 0.80,
            DuplicationType::SemanticClone => 0.70,
        }
    }
}

/// Configuration thresholds for duplication detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationThresholds {
    /// Minimum number of lines for a clone to be reported
    pub min_lines: usize,
    /// Minimum number of tokens for a clone to be reported
    pub min_tokens: usize,
    /// Similarity threshold (0.0 - 1.0) for considering code as duplicate
    pub similarity_threshold: f64,
    /// Enable Type 1 (exact) clone detection
    pub detect_exact: bool,
    /// Enable Type 2 (renamed) clone detection
    pub detect_renamed: bool,
    /// Enable Type 3 (gapped) clone detection
    pub detect_gapped: bool,
    /// Enable Type 4 (semantic) clone detection (experimental)
    pub detect_semantic: bool,
    /// Languages to analyze
    pub languages: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum gap size for gapped clones (number of different tokens)
    pub max_gap_size: usize,
}

impl Default for DuplicationThresholds {
    fn default() -> Self {
        Self {
            min_lines: 6,
            min_tokens: 50,
            similarity_threshold: 0.80,
            detect_exact: true,
            detect_renamed: true,
            detect_gapped: true,
            detect_semantic: false, // Disabled by default (experimental)
            languages: vec![
                "rust".to_owned(),
                "python".to_owned(),
                "javascript".to_owned(),
                "typescript".to_owned(),
            ],
            exclude_patterns: vec![
                "**/target/**".to_owned(),
                "**/node_modules/**".to_owned(),
                "**/.git/**".to_owned(),
                "**/vendor/**".to_owned(),
            ],
            max_gap_size: 5,
        }
    }
}

impl DuplicationThresholds {
    /// Create thresholds for strict detection (higher sensitivity)
    #[must_use]
    pub fn strict() -> Self {
        Self {
            min_lines: 4,
            min_tokens: 30,
            similarity_threshold: 0.90,
            ..Default::default()
        }
    }

    /// Create thresholds for lenient detection (lower sensitivity)
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            min_lines: 10,
            min_tokens: 100,
            similarity_threshold: 0.70,
            ..Default::default()
        }
    }

    /// Check if a duplication type should be detected based on thresholds
    #[must_use]
    pub fn should_detect(&self, dup_type: DuplicationType) -> bool {
        match dup_type {
            DuplicationType::ExactClone => self.detect_exact,
            DuplicationType::RenamedClone => self.detect_renamed,
            DuplicationType::GappedClone => self.detect_gapped,
            DuplicationType::SemanticClone => self.detect_semantic,
        }
    }

    /// Check if a similarity value meets the threshold for a given type
    #[must_use]
    pub fn meets_threshold(&self, similarity: f64, dup_type: DuplicationType) -> bool {
        let type_min = dup_type.min_similarity();
        similarity >= self.similarity_threshold.max(type_min)
    }
}
