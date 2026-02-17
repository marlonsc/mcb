#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::value_objects::SupportedLanguage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file: String,
    pub language: Option<SupportedLanguage>,
    pub cyclomatic: f64,
    pub cognitive: f64,
    pub maintainability_index: f64,
    pub sloc: usize,
    pub ploc: usize,
    pub lloc: usize,
    pub cloc: usize,
    pub blank: usize,
    pub halstead: Option<HalsteadMetrics>,
}

impl Default for FileMetrics {
    fn default() -> Self {
        Self {
            file: String::new(),
            language: None,
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            ploc: 0,
            lloc: 0,
            cloc: 0,
            blank: 0,
            halstead: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    pub n1: usize,
    pub n2: usize,
    pub n1_total: usize,
    pub n2_total: usize,
    pub vocabulary: usize,
    pub length: usize,
    pub calculated_length: f64,
    pub volume: f64,
    pub difficulty: f64,
    pub effort: f64,
    pub bugs: f64,
    pub time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub cyclomatic: f64,
    pub cognitive: f64,
    pub sloc: usize,
    pub parameters: usize,
    pub nesting_depth: usize,
}
