//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! AST-based Unwrap Detector using rust-code-analysis
//!
//! Uses our fork with extended Node API for unwrap/expect detection.
//! Replaces the tree-sitter direct implementation with RCA Callback pattern.

use std::path::Path;

use rust_code_analysis::{Callback, Node, ParserTrait, action, guess_language};

use crate::constants::common::{CFG_TEST_MARKER, EXPECT_CALL, MOD_PREFIX, UNWRAP_CALL};
use crate::{Result, ValidationError};

/// Detection result for unwrap/expect usage
#[derive(Debug, Clone)]
pub struct UnwrapDetection {
    /// File where the detection occurred
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// The specific method detected ("unwrap", "expect")
    pub method: String,
    /// Whether this is in a test module
    pub in_test: bool,
    /// The source text of the method call
    pub context: String,
}

/// Configuration for unwrap detection callback
struct UnwrapConfig {
    filename: String,
    test_ranges: Vec<(usize, usize)>,
}

/// RCA Callback for unwrap detection
struct UnwrapCallback;

impl Callback for UnwrapCallback {
    type Res = Vec<UnwrapDetection>;
    type Cfg = UnwrapConfig;

    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        let root = parser.get_root();
        let code = parser.get_code();
        let mut detections = Vec::new();

        // Recursive detection through AST
        detect_recursive(&root, code, &cfg, &mut detections);
        detections
    }
}

fn detect_recursive(
    node: &Node,
    code: &[u8],
    cfg: &UnwrapConfig,
    results: &mut Vec<UnwrapDetection>,
) {
    let mut stack = vec![node.0];

    while let Some(ts_node) = stack.pop() {
        let current = Node(ts_node);

        if current.kind() == "call_expression"
            && let Some(text) = current.utf8_text(code)
        {
            let method = extract_method(text);
            if matches!(method.as_str(), "unwrap" | "expect") {
                let byte_pos = current.start_byte();
                let in_test = cfg
                    .test_ranges
                    .iter()
                    .any(|(start, end)| byte_pos >= *start && byte_pos < *end);

                results.push(UnwrapDetection {
                    file: cfg.filename.clone(),
                    line: current.start_row() + 1,
                    column: current.start_position().1 + 1,
                    method,
                    in_test,
                    context: text.lines().next().unwrap_or("").trim().to_owned(),
                });
            }
        }

        let mut cursor = ts_node.walk();
        let children: Vec<_> = ts_node.children(&mut cursor).collect();
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
}

/// Extract method name from call expression text
fn extract_method(text: &str) -> String {
    if text.contains(UNWRAP_CALL) {
        "unwrap".to_owned()
    } else if text.contains(EXPECT_CALL) {
        "expect".to_owned()
    } else {
        String::new()
    }
}

/// Find test module ranges in the AST
fn find_test_ranges(root: &Node, code: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    find_test_modules_recursive(root, code, &mut ranges);
    ranges
}

fn find_test_modules_recursive(node: &Node, code: &[u8], ranges: &mut Vec<(usize, usize)>) {
    let mut stack = vec![node.0];

    while let Some(ts_node) = stack.pop() {
        let current = Node(ts_node);

        if current.kind() == "mod_item" {
            let start = current.start_byte();
            let search_start = start.saturating_sub(50);
            let before = std::str::from_utf8(&code[search_start..start]).unwrap_or("");
            if before.contains(CFG_TEST_MARKER) {
                ranges.push((current.start_byte(), current.end_byte()));
                continue;
            }

            if let Some(name_text) = current.utf8_text(code)
                && (name_text.contains(&format!("{MOD_PREFIX}tests"))
                    || name_text.contains(&format!("{MOD_PREFIX}test")))
            {
                let all_before = std::str::from_utf8(&code[..start]).unwrap_or("");
                if let Some(attr_pos) = all_before.rfind(CFG_TEST_MARKER) {
                    let between = &all_before[attr_pos..];
                    if !between.contains(MOD_PREFIX)
                        || between.rfind(MOD_PREFIX).unwrap_or(0) == between.len() - name_text.len()
                    {
                        ranges.push((current.start_byte(), current.end_byte()));
                        continue;
                    }
                }
            }
        }

        let mut cursor = ts_node.walk();
        let children: Vec<_> = ts_node.children(&mut cursor).collect();
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
}

/// RCA Callback for finding test module ranges
struct TestRangeCallback;

impl Callback for TestRangeCallback {
    type Res = Vec<(usize, usize)>;
    type Cfg = ();

    fn call<T: ParserTrait>(_cfg: (), parser: &T) -> Self::Res {
        find_test_ranges(&parser.get_root(), parser.get_code())
    }
}

/// Detect unwrap/expect in file content
///
/// # Errors
///
/// Returns an error if the language cannot be determined from the filename.
pub fn detect_in_content(content: &str, filename: &str) -> Result<Vec<UnwrapDetection>> {
    let path = Path::new(filename);
    let source = content.as_bytes().to_vec();

    let (lang, _) = guess_language(&source, path);
    let lang = lang.ok_or_else(|| {
        ValidationError::Config(format!("Unsupported language for file: {filename}"))
    })?;

    // First pass: find test module ranges
    let test_ranges = action::<TestRangeCallback>(&lang, source.clone(), path, None, ());

    // Second pass: detect unwraps
    let cfg = UnwrapConfig {
        filename: filename.to_owned(),
        test_ranges,
    };

    Ok(action::<UnwrapCallback>(&lang, source, path, None, cfg))
}

/// Detect unwrap/expect in file
///
/// # Errors
///
/// Returns an error if the file cannot be read or its language is unsupported.
pub fn detect_in_file(path: &Path) -> Result<Vec<UnwrapDetection>> {
    let content = std::fs::read_to_string(path)?;
    let file_name = path
        .to_str()
        .ok_or_else(|| ValidationError::Config(format!("Non-UTF8 path: {}", path.display())))?;
    detect_in_content(&content, file_name)
}

/// AST-based unwrap detector using rust-code-analysis
///
/// Provides the same API as before but uses RCA internally.
pub struct UnwrapDetector;

impl UnwrapDetector {
    /// Create a new unwrap detector
    ///
    /// # Errors
    ///
    /// This constructor is infallible but returns `Result` for API consistency.
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Detect unwrap/expect calls in Rust source code
    ///
    /// # Errors
    ///
    /// Returns an error if the language cannot be determined from the filename.
    pub fn detect_in_content(
        &mut self,
        content: &str,
        filename: &str,
    ) -> Result<Vec<UnwrapDetection>> {
        detect_in_content(content, filename)
    }

    /// Detect unwrap/expect calls in a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or its language is unsupported.
    pub fn detect_in_file(&mut self, path: &Path) -> Result<Vec<UnwrapDetection>> {
        detect_in_file(path)
    }
}

impl Default for UnwrapDetector {
    fn default() -> Self {
        Self
    }
}
