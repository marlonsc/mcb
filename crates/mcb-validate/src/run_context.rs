//! Execution context for validation runs.
//!
//! Provides the environment, file inventory, and caching for a single validation execution.
//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use walkdir::WalkDir;

use crate::config::FileConfig;
use crate::filters::{LanguageDetector, LanguageId};
use crate::{Result, ValidationConfig};

/// Source used to build the file inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInventorySource {
    /// Inventory built using `git ls-files`
    Git,
    /// Inventory built by walking the directory recursively
    WalkDir,
}

impl FileInventorySource {
    /// Return the string representation of the source
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::WalkDir => "walkdir",
        }
    }
}

/// A single file entry in the validation inventory.
#[derive(Debug, Clone)]
pub struct InventoryEntry {
    /// Absolute path to the file on disk
    pub absolute_path: PathBuf,
    /// Path relative to the workspace root
    pub relative_path: PathBuf,
    /// Language detected during inventory building (single-pass).
    pub detected_language: Option<LanguageId>,
}

/// Context for a single validation run.
///
/// Contains the workspace layout, file inventory, and shared caches used during validation.
#[derive(Debug)]
pub struct ValidationRunContext {
    workspace_root: PathBuf,
    trace_id: String,
    file_inventory: Arc<Vec<InventoryEntry>>,
    file_inventory_source: FileInventorySource,
    content_cache: Mutex<HashMap<PathBuf, Arc<str>>>,
}

thread_local! {
    static ACTIVE_RUN_CONTEXT: RefCell<Option<Arc<ValidationRunContext>>> = const { RefCell::new(None) };
}

impl ValidationRunContext {
    /// Create a new validation context.
    ///
    /// # Errors
    /// Returns an error if file inventory enumeration fails.
    pub fn build(config: &ValidationConfig) -> Result<Self> {
        let file_config = FileConfig::load(&config.workspace_root);
        let mut ignore_patterns = file_config.general.exclude_patterns.clone();
        ignore_patterns.extend(config.exclude_patterns.iter().cloned());

        let (entries, source) = enumerate_inventory(&config.workspace_root, &ignore_patterns)?;

        Ok(Self {
            workspace_root: config.workspace_root.clone(),
            trace_id: build_trace_id(),
            file_inventory: Arc::new(entries),
            file_inventory_source: source,
            content_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Get the absolute path to the workspace root
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get the unique trace identifier for this run
    #[must_use]
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the full list of files in the inventory
    #[must_use]
    pub fn file_inventory(&self) -> &[InventoryEntry] {
        &self.file_inventory
    }

    /// Get the source used to build the inventory
    #[must_use]
    pub fn file_inventory_source(&self) -> FileInventorySource {
        self.file_inventory_source
    }

    /// Get total number of files in the inventory
    #[must_use]
    pub fn file_inventory_count(&self) -> usize {
        self.file_inventory.len()
    }

    /// Check if the inventory contains any files for the given language
    #[must_use]
    pub fn has_files_for_language(&self, lang: LanguageId) -> bool {
        self.file_inventory
            .iter()
            .any(|e| e.detected_language == Some(lang))
    }

    /// Get all files in the inventory that match the given language
    #[must_use]
    pub fn files_for_language(&self, lang: LanguageId) -> Vec<&InventoryEntry> {
        self.file_inventory
            .iter()
            .filter(|e| e.detected_language == Some(lang))
            .collect()
    }

    /// Get all files in the inventory that match any of the given languages
    #[must_use]
    pub fn files_matching_languages(&self, langs: &[LanguageId]) -> Vec<&InventoryEntry> {
        self.file_inventory
            .iter()
            .filter(|e| e.detected_language.is_some_and(|l| langs.contains(&l)))
            .collect()
    }

    /// Convenience method to get all Rust files in the inventory
    #[must_use]
    pub fn rs_files(&self) -> Vec<&InventoryEntry> {
        self.files_for_language(LanguageId::Rust)
    }

    /// Read file content, using cache if available.
    ///
    /// # Errors
    /// Returns an error if file reading fails.
    pub fn read_cached(&self, path: &Path) -> std::io::Result<Arc<str>> {
        let normalized = normalize_path(path)?;
        if let Ok(cache) = self.content_cache.lock()
            && let Some(content) = cache.get(&normalized)
        {
            return Ok(Arc::clone(content));
        }

        let content = std::fs::read_to_string(&normalized)?;
        let value: Arc<str> = Arc::from(content);

        if let Ok(mut cache) = self.content_cache.lock() {
            cache.insert(normalized, Arc::clone(&value));
        }

        Ok(value)
    }

    /// Execute a closure with the given context set as active for the current thread
    pub fn with_active<T>(context: &Arc<Self>, f: impl FnOnce() -> T) -> T {
        ACTIVE_RUN_CONTEXT.with(|slot| {
            let previous = slot.replace(Some(Arc::clone(context)));
            let output = f();
            slot.replace(previous);
            output
        })
    }

    /// Get the active validation context for the current thread
    #[must_use]
    pub fn active() -> Option<Arc<Self>> {
        ACTIVE_RUN_CONTEXT.with(|slot| slot.borrow().as_ref().map(Arc::clone))
    }

    /// Get the active context or build a new one.
    ///
    /// # Errors
    /// Returns an error if the context needs to be built and it fails.
    pub fn active_or_build(config: &ValidationConfig) -> Result<Arc<Self>> {
        if let Some(active) = Self::active() {
            return Ok(active);
        }

        Ok(Arc::new(Self::build(config)?))
    }
}

fn enumerate_inventory(
    workspace_root: &Path,
    ignore_patterns: &[String],
) -> Result<(Vec<InventoryEntry>, FileInventorySource)> {
    let detector = LanguageDetector::new();

    if workspace_root.join(".git").exists()
        && let Ok(Some(entries)) = enumerate_with_git(workspace_root, ignore_patterns, &detector)
    {
        return Ok((entries, FileInventorySource::Git));
    }

    let entries = enumerate_with_walkdir(workspace_root, ignore_patterns, &detector)?;
    Ok((entries, FileInventorySource::WalkDir))
}

fn enumerate_with_git(
    workspace_root: &Path,
    ignore_patterns: &[String],
    detector: &LanguageDetector,
) -> std::io::Result<Option<Vec<InventoryEntry>>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .arg("ls-files")
        .arg("-co")
        .arg("--exclude-standard")
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }

        let relative = PathBuf::from(line);
        let absolute = workspace_root.join(&relative);

        if !absolute.is_file() {
            continue;
        }

        if should_ignore(line, ignore_patterns) {
            continue;
        }

        if seen.insert(relative.clone()) {
            let abs = normalize_path(&absolute)?;
            let lang = detector.detect(&abs, None);
            entries.push(InventoryEntry {
                absolute_path: abs,
                relative_path: relative,
                detected_language: lang,
            });
        }
    }

    Ok(Some(entries))
}

fn enumerate_with_walkdir(
    workspace_root: &Path,
    ignore_patterns: &[String],
    detector: &LanguageDetector,
) -> std::io::Result<Vec<InventoryEntry>> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();

    // Canonicalize workspace_root so strip_prefix works consistently with
    // canonicalized file paths. On macOS /tmp → /private/tmp symlink and on
    // Windows the \\?\ prefix would otherwise cause strip_prefix to fail.
    let canonical_root = normalize_path(workspace_root)?;

    for entry in WalkDir::new(&canonical_root)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let absolute = normalize_path(path)?;
        let Ok(relative) = absolute.strip_prefix(&canonical_root) else {
            continue;
        };
        let relative = relative.to_path_buf();

        let Some(relative_str) = relative.to_str() else {
            continue;
        };
        if should_ignore(relative_str, ignore_patterns)
            || relative_str.contains("/.git/")
            || relative_str.starts_with(".git/")
        {
            continue;
        }

        if seen.insert(relative.clone()) {
            let lang = detector.detect(&absolute, None);
            entries.push(InventoryEntry {
                absolute_path: absolute,
                relative_path: relative,
                detected_language: lang,
            });
        }
    }

    Ok(entries)
}

fn should_ignore(path: &str, ignore_patterns: &[String]) -> bool {
    ignore_patterns.iter().any(|pattern| path.contains(pattern))
}

fn build_trace_id() -> String {
    let nanos = mcb_domain::utils::time::epoch_nanos_u128().unwrap_or(0);
    format!("validate-run-{nanos}")
}

fn normalize_path(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}
