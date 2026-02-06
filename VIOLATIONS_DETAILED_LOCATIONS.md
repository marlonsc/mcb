# Detailed Violation Locations & Code References

## Issue #1: PAT004 - BrowseResult Definition (FALSE POSITIVE)

**File**: `crates/mcb-domain/src/ports/browse.rs`

**Reported Location**: Line 27

**Actual Content**:
```rust
// Line 27 (actual):
pub enum HighlightError {
    /// Invalid configuration for highlighting
    #[error("Highlighting configuration error: {0}")]
    ConfigurationError(String),
    // ...
}
```

**Finding**: No `BrowseResult` type alias exists. The file only defines error types and service traits.

---

## Issue #2: IMPL001 - walk_directory_boxed (LIKELY FALSE POSITIVE)

**File**: `crates/mcb-server/src/handlers/browse_service.rs`

**Reported Location**: Line 71

**Actual Code** (lines 55-111):
```rust
fn walk_directory_boxed(
    &self,
    path: &Path,
    depth: usize,
    max_depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FileNode>> + Send + '_>> {
    let path = path.to_path_buf();
    Box::pin(async move {
        let metadata = tokio::fs::metadata(&path).await.map_err(BrowseError::Io)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string();

        let is_dir = metadata.is_dir();
        let language = if is_dir { None } else { detect_language(&path) };

        let children = if is_dir && depth < max_depth {
            let mut entries = tokio::fs::read_dir(&path).await.map_err(BrowseError::Io)?;
            let mut children = Vec::new();

            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();

                if should_skip_path(&entry_path) {
                    continue;
                }

                match self
                    .walk_directory_boxed(&entry_path, depth + 1, max_depth)
                    .await
                {
                    Ok(node) => children.push(node),
                    Err(_) => continue,
                }
            }

            if children.is_empty() {
                None
            } else {
                Some(children)
            }
        } else {
            None
        };

        Ok(FileNode {
            path,
            name,
            is_dir,
            children,
            language,
            lines: None,
        })
    })
}
```

**Finding**: Method is fully implemented. The `children` field being `None` is intentional for leaf nodes.

---

## Issue #3: IMPL006 - Empty Catch-All (FALSE POSITIVE)

**File**: `crates/mcb-server/src/handlers/highlight_service.rs`

**Reported Location**: Line 125

**Actual Code** (lines 120-127):
```rust
"swift" => Ok(HighlightLanguageConfig::new(
    "swift",
    tree_sitter_swift::LANGUAGE.into(),
    tree_sitter_swift::HIGHLIGHTS_QUERY,
)),
_ => Err(HighlightError::UnsupportedLanguage(language.to_string())),
}
```

**Finding**: Line 125 contains proper error handling, not an empty catch-all.

---

## Issue #4 & #5: PERF002 + PERF001 - Allocations & Clones in Loop (NEEDS FIX)

**File**: `crates/mcb-server/src/admin/browse_handlers.rs`

**Reported Locations**: Lines 220, 221, 222

**Actual Code** (lines 206-224):
```rust
206 |     let mut chunk_responses = Vec::with_capacity(chunks.len());
207 |     for c in chunks {
208 |         // Estimate end line from content
209 |         let line_count = c.content.lines().count() as u32;
210 |         let end_line = c.start_line.saturating_add(line_count.saturating_sub(1));
211 |
212 |         // Generate server-side highlighting via injected service
213 |         let highlighted = match state
214 |             .highlight_service
215 |             .highlight(&c.content, &c.language)
216 |             .await
217 |         {
218 |             Ok(h) => h,
219 |             Err(_) => mcb_domain::value_objects::browse::HighlightedCode::new(
220 |                 c.content.clone(),        // ← PERF001: Clone in loop
221 |                 Vec::new(),                // ← PERF002: Allocation in loop
222 |                 c.language.clone(),       // ← PERF001: Clone in loop
223 |             ),
224 |         };
```

**Issues**:
1. Line 220: `c.content.clone()` - String clone in loop
2. Line 221: `Vec::new()` - Vector allocation in loop
3. Line 222: `c.language.clone()` - String clone in loop

**Recommended Fix**:
```rust
let empty_spans = Vec::new();
let mut chunk_responses = Vec::with_capacity(chunks.len());
for c in chunks {
    // ... code ...
    Err(_) => mcb_domain::value_objects::browse::HighlightedCode::new(
        c.content.clone(),
        empty_spans.clone(),  // Reuse instead of allocating
        c.language.clone(),
    ),
}
```

---

## Issue #6: CA005 - Mutable Method in Value Object (NEEDS FIX)

**File**: `crates/mcb-domain/src/value_objects/browse/tree.rs`

**Reported Location**: Line 52

**Actual Code** (lines 51-60):
```rust
51 |     /// Add a child node to this directory (in-place)
52 |     pub fn add_child(&mut self, child: FileTreeNode) {
53 |         self.children.push(child);
54 |     }
55 |
56 |     /// Add a child node to this directory (builder pattern)
57 |     pub fn with_child(mut self, child: FileTreeNode) -> Self {
58 |         self.children.push(child);
59 |         self
60 |     }
```

**Issue**: Value objects should be immutable. The `add_child(&mut self)` method violates this principle.

**Current Usage** (browse_handlers.rs:304):
```rust
304 |         node.add_child(file_node);
```

**Recommended Fix**:
1. Remove `add_child(&mut self)` method (lines 51-54)
2. Update usage to use builder pattern:
   ```rust
   node = node.with_child(file_node);
   ```

---

## Issue #7: ORG016 - Domain Layer Method Violation (NEEDS INVESTIGATION)

**File**: `crates/mcb-domain/src/entities/vcs/vcs_repo.rs`

**Reported Location**: Line 32

**Actual Code** (lines 31-34):
```rust
31 |     #[must_use]
32 |     pub fn id(&self) -> &RepositoryId {
33 |         &self.id
34 |     }
```

**Related Violations** (similar pattern):
- `crates/mcb-domain/src/entities/vcs/branch.rs:29` - `VcsBranch::id()`
- `crates/mcb-domain/src/entities/vcs/branch.rs:33` - `VcsBranch::name()`
- `crates/mcb-domain/src/entities/vcs/branch.rs:37` - `VcsBranch::head_commit()`

**Issue**: Validator claims domain entities should only have trait definitions, not concrete methods.

**Question**: Are getter methods allowed in domain entities, or should all access go through traits?

---

## Issue #8: QUAL004 - Large File (ALREADY FIXED)

**File**: `crates/mcb-providers/src/database/sqlite/project_repository/`

**Reported Issue**: File has 559 lines

**Actual Structure** (already modularized):
```
project_repository/
├── mod.rs          (132 lines) - Main repository implementation
├── decision.rs     (2460 bytes) - Decision operations
├── dependency.rs   (2102 bytes) - Dependency operations
├── issue.rs        (6279 bytes) - Issue operations
├── phase.rs        (3397 bytes) - Phase operations
└── project.rs      (3809 bytes) - Project operations
```

**Finding**: File is already properly split into logical modules.

---

## Result Type Aliases Verification

### Correct Usage (using domain Error types):

**File**: `crates/mcb-domain/src/error.rs`
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

**File**: `crates/mcb-validate/src/lib.rs`
```rust
pub type Result<T> = std::result::Result<T, ValidationError>;
```

**File**: `crates/mcb-ast-utils/src/error.rs`
```rust
pub type Result<T> = std::result::Result<T, AstError>;
```

**File**: `crates/mcb-language-support/src/error.rs`
```rust
pub type Result<T> = std::result::Result<T, LanguageError>;
```

### Questionable Usage:

**File**: `crates/mcb-providers/src/workflow/transitions.rs`
```rust
type Result<T> = std::result::Result<T, String>;  // ⚠️ Should use domain Error
```

### Test-Specific (OK):

**File**: `crates/mcb-providers/tests/unit/git2_provider_tests.rs`
```rust
type TestResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
```

---

## Summary of Actionable Items

| Priority | Issue | File | Lines | Action |
|----------|-------|------|-------|--------|
| HIGH | CA005 | tree.rs | 52-54 | Remove `add_child()`, use `with_child()` |
| MEDIUM | PERF002 | browse_handlers.rs | 221 | Move `Vec::new()` outside loop |
| MEDIUM | PERF001 | browse_handlers.rs | 220, 222 | Reduce clones in loop |
| LOW | ORG016 | vcs_repo.rs, branch.rs | 32, 29, 33, 37 | Clarify getter method rules |
| NONE | PAT004 | browse.rs | 27 | False positive - ignore |
| NONE | IMPL001 | browse_service.rs | 71 | Likely false positive - verify |
| NONE | IMPL006 | highlight_service.rs | 125 | False positive - ignore |
| NONE | QUAL004 | project_repository/ | - | Already fixed - ignore |

