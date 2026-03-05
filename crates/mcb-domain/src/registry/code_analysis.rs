//! Code analysis registry.

use std::sync::Arc;

use crate::error::Result;
use crate::ports::providers::analysis::CodeAnalyzer;

/// Registry entry for code analyzers.
#[doc(hidden)]
pub struct CodeAnalyzerEntry {
    /// Unique name of the analyzer.
    pub name: &'static str,
    /// Human-readable description of what it detects.
    pub description: &'static str,
    /// Factory function to build the analyzer instance.
    pub build: fn() -> Result<Arc<dyn CodeAnalyzer>>,
}

/// Distributed slice of registered code analyzers.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static CODE_ANALYZERS: [CodeAnalyzerEntry] = [..];

/// Resolve a code analyzer by name from the registry.
///
/// # Errors
/// Returns an error if the named analyzer is not registered or its build function fails.
pub fn resolve_code_analyzer(name: &str) -> Result<Arc<dyn CodeAnalyzer>> {
    for entry in CODE_ANALYZERS.iter() {
        if entry.name == name {
            return (entry.build)().map_err(|e| {
                crate::error::Error::configuration(format!("code analyzer '{name}': {e}"))
            });
        }
    }

    let available: Vec<&str> = CODE_ANALYZERS.iter().map(|e| e.name).collect();
    Err(crate::error::Error::configuration(format!(
        "Unknown code analyzer '{name}'. Available: {available:?}"
    )))
}

/// Resolve the first available code analyzer from the registry.
///
/// # Errors
/// Returns an error if no analyzers are registered or the build function fails.
pub fn resolve_default_code_analyzer() -> Result<Arc<dyn CodeAnalyzer>> {
    let entry = CODE_ANALYZERS.iter().next().ok_or_else(|| {
        crate::error::Error::configuration("No code analyzers registered".to_owned())
    })?;
    (entry.build)().map_err(|e| {
        crate::error::Error::configuration(format!("code analyzer '{}': {}", entry.name, e))
    })
}

/// List all registered code analyzers as `(name, description)` pairs.
#[must_use]
pub fn list_code_analyzers() -> Vec<(&'static str, &'static str)> {
    CODE_ANALYZERS
        .iter()
        .map(|e| (e.name, e.description))
        .collect()
}
