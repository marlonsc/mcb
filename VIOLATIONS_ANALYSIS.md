# Architecture & Quality Violations Analysis

**Generated**: 2026-02-06  
**Validation Report**: 329 total violations (1 error, 204 warnings, 124 infos)

---

## 1. PAT004: BrowseResult Definition Issue

### Status: ✅ RESOLVED - No violation found

**Report Claim**: "BrowseResult defined at mcb-domain/src/ports/browse.rs:27"

**Finding**: The file `/home/marlonsc/mcb/crates/mcb-domain/src/ports/browse.rs` does **NOT** contain a `BrowseResult` type alias at line 27 or anywhere in the file.

**Actual Content** (lines 1-72):

-   Line 27 contains: `pub enum HighlightError {` (error type definition)
-   No `BrowseResult` type alias exists in this file
-   The file defines two error types: `BrowseError` and `HighlightError`
-   The file defines two service traits: `BrowseService` and `HighlightService`

**Conclusion**: This is a **false positive** in the validation report. The validator may have been looking for a `Result` type alias that doesn't exist. The codebase correctly uses `crate::Result<T>` from `mcb-domain/src/error.rs` which is defined as:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

**Action**: No fix needed. This violation can be safely ignored or the validator rule should be updated.

---

## 2. IMPL001: walk_directory_boxed Empty Method Body

### Status: ⚠️ NEEDS INVESTIGATION

**Location**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/browse_service.rs:71`

**Report Claim**: "walk_directory_boxed() returns None"

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

**Analysis**:

-   The method is **NOT empty** - it has a complete implementation
-   It returns a boxed async future that produces `Result<FileNode>`
-   The `children` field can be `None` (lines 93-99), which is intentional for leaf nodes
-   The validator may be incorrectly flagging the `None` return as "empty"

**Conclusion**: This is likely a **false positive** or misleading violation message. The method is fully implemented and returns proper `FileNode` structures.

**Action**: Verify with validator author or suppress this rule if it's a known false positive.

---

## 3. IMPL006: Empty Catch-All Pattern in highlight_service.rs

### Status: ✅ RESOLVED - No violation found

**Report Claim**: "Empty catch-all '_=> {}' in mcb-server/src/handlers/highlight_service.rs:125"

**Actual Code** (lines 120-127):

```rust
"swift" => Ok(HighlightLanguageConfig::new(
    "swift",
    tree_sitter_swift::LANGUAGE.into(),
    tree_sitter_swift::HIGHLIGHTS_QUERY,
)),
_ => Err(HighlightError::UnsupportedLanguage(language.to_string())),
```

**Finding**: Line 125 contains a proper error return, **NOT** an empty catch-all pattern.

**Conclusion**: This is a **false positive**. The catch-all pattern properly returns an error for unsupported languages.

**Action**: No fix needed.

---

## 4. PERF002: Allocation in Loop

### Status: ⚠️ NEEDS FIX

**Location**: `/home/marlonsc/mcb/crates/mcb-server/src/admin/browse_handlers.rs:221`

**Report Claim**: "Vec::new() allocation in loop"

**Actual Code** (lines 206-224):

```rust
let mut chunk_responses = Vec::with_capacity(chunks.len());
for c in chunks {
    // Estimate end line from content
    let line_count = c.content.lines().count() as u32;
    let end_line = c.start_line.saturating_add(line_count.saturating_sub(1));

    // Generate server-side highlighting via injected service
    let highlighted = match state
        .highlight_service
        .highlight(&c.content, &c.language)
        .await
    {
        Ok(h) => h,
        Err(_) => mcb_domain::value_objects::browse::HighlightedCode::new(
            c.content.clone(),
            Vec::new(),  // ← LINE 221: Allocation in loop
            c.language.clone(),
        ),
    };
```

**Issue**: `Vec::new()` is allocated inside the loop on line 221 as part of the error fallback path.

**Related Issues**:

-   **PERF001** (line 220): `c.content.clone()` - Clone in loop
-   **PERF001** (line 222): `c.language.clone()` - Clone in loop

**Recommendation**:

1.  Move the empty `Vec::new()` outside the loop or create it once
2.  Consider using references instead of cloning `content` and `language`
3.  Or create a helper function to avoid repeated allocations

**Fix Strategy**:

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

## 5. CA005: Mutable Method in Value Object

### Status: ⚠️ NEEDS FIX

**Location**: `/home/marlonsc/mcb/crates/mcb-domain/src/value_objects/browse/tree.rs:52`

**Report Claim**: "Value object FileTreeNode has mutable method add_child"

**Actual Code** (lines 51-60):

```rust
/// Add a child node to this directory (in-place)
pub fn add_child(&mut self, child: FileTreeNode) {
    self.children.push(child);
}

/// Add a child node to this directory (builder pattern)
pub fn with_child(mut self, child: FileTreeNode) -> Self {
    self.children.push(child);
    self
}
```

**Issue**: Value objects should be immutable. The `add_child(&mut self)` method violates this principle.

**Current Usage** (browse_handlers.rs:304):

```rust
node.add_child(file_node);
```

**Recommendation**: Replace mutable method with builder pattern:

1.  Remove `add_child(&mut self)` method
2.  Use `with_child(self, child)` (already exists) for building
3.  Update all call sites to use builder pattern

**Fix Strategy**:

```rust
// Instead of:
node.add_child(file_node);

// Use:
node = node.with_child(file_node);
```

---

## 6. ORG016: Domain Layer Method Violation

### Status: ⚠️ NEEDS INVESTIGATION

**Location**: `/home/marlonsc/mcb/crates/mcb-domain/src/entities/vcs/vcs_repo.rs:32`

**Report Claim**: "Domain layer has method for VcsRepository::id (domain should be trait-only)"

**Actual Code** (lines 31-34):

```rust
#[must_use]
pub fn id(&self) -> &RepositoryId {
    &self.id
}
```

**Issue**: The validator claims domain entities should only have trait definitions, not concrete methods.

**Analysis**:

-   This is a getter method for a private field
-   It's a simple accessor, not business logic
-   The `#[must_use]` attribute suggests it's intentional
-   Similar violations exist for `VcsBranch::id`, `VcsBranch::name`, `VcsBranch::head_commit`

**Architectural Question**:

-   Is the rule too strict? Getter methods are typically allowed in domain entities
-   Or should all access go through traits?

**Recommendation**:

1.  Clarify the architectural rule in ADRs
2.  If getters are allowed, update the validator rule
3.  If not allowed, create a trait and move methods there

---

## 7. QUAL004: Large File - project_repository

### Status: ✅ ALREADY SPLIT

**Location**: `/home/marlonsc/mcb/crates/mcb-providers/src/database/sqlite/project_repository/`

**Report Claim**: "project_repository.rs has 559 lines"

**Actual Structure**: The file has already been split into modules:

```
project_repository/
├── mod.rs          (132 lines) - Main repository implementation
├── decision.rs     (2460 bytes) - Decision operations
├── dependency.rs   (2102 bytes) - Dependency operations
├── issue.rs        (6279 bytes) - Issue operations
├── phase.rs        (3397 bytes) - Phase operations
└── project.rs      (3809 bytes) - Project operations
```

**Finding**: The file is already properly modularized. The validator may be looking at an older version or aggregating the module size.

**Conclusion**: No action needed. The code is already well-organized.

---

## 8. Result Type Aliases Using std::Result::Result

### Status: ✅ VERIFIED - Correct Usage

**Search Results**: Found 6 Result type aliases in the codebase:

1.  **mcb-domain/src/error.rs** ✅

   ```rust
   pub type Result<T> = std::result::Result<T, Error>;
   ```

-   Correct: Uses domain Error type

1.  **mcb-providers/src/workflow/transitions.rs** ⚠️

   ```rust
   type Result<T> = std::result::Result<T, String>;
   ```

-   Uses String error type (should use domain Error)

1.  **mcb-validate/src/lib.rs** ✅

   ```rust
   pub type Result<T> = std::result::Result<T, ValidationError>;
   ```

-   Correct: Uses ValidationError type

1.  **mcb-ast-utils/src/error.rs** ✅

   ```rust
   pub type Result<T> = std::result::Result<T, AstError>;
   ```

-   Correct: Uses AstError type

1.  **mcb-language-support/src/error.rs** ✅

   ```rust
   pub type Result<T> = std::result::Result<T, LanguageError>;
   ```

-   Correct: Uses LanguageError type

1.  **mcb-providers/tests/unit/git2_provider_tests.rs** ✅

   ```rust
   type TestResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
   ```

-   Correct: Test-specific Result type

**Finding**: No `BrowseResult` or `HighlightResult` type aliases found. The codebase correctly uses `crate::Result<T>` from mcb-domain.

---

## Summary of Violations

| ID | Category | Severity | Status | Action |
|---|---|---|---|---|
| PAT004 | Pattern | Info | False Positive | Ignore |
| IMPL001 | Implementation | Warning | False Positive | Ignore |
| IMPL006 | Implementation | Warning | False Positive | Ignore |
| PERF002 | Performance | Warning | **Needs Fix** | Move Vec::new() outside loop |
| PERF001 | Performance | Warning | **Needs Fix** | Reduce clones in loop |
| CA005 | Architecture | Warning | **Needs Fix** | Remove mutable add_child() |
| ORG016 | Organization | Warning | Investigate | Clarify getter method rules |
| QUAL004 | Quality | Info | Already Fixed | No action needed |

---

## Recommended Fix Priority

### High Priority (Breaking Architecture)

1.  **CA005**: Remove mutable `add_child()` method from FileTreeNode

-   Affects: browse_handlers.rs (1 location)
-   Effort: Low (simple refactor)

### Medium Priority (Performance)

1.  **PERF002 + PERF001**: Reduce allocations and clones in browse_handlers.rs:206-224

-   Affects: browse_handlers.rs (1 location)
-   Effort: Low (move allocation, consider references)

### Low Priority (Clarification)

1.  **ORG016**: Clarify domain entity getter method rules

-   Affects: vcs_repo.rs, branch.rs (multiple locations)
-   Effort: Medium (requires architectural decision)

### False Positives (No Action)

-   PAT004, IMPL001, IMPL006, QUAL004

---

## Validation Report Statistics

```
Total Violations: 329
├── Errors: 1 (PMAT005 - dead code analysis timeout)
├── Warnings: 204
└── Infos: 124

By Category:
├── Documentation: 108 violations
├── KISS: 79 violations
├── Error Boundary: 50 violations
├── SOLID: 25 violations
├── Organization: 26 violations
├── Testing: 24 violations
├── Refactoring: 9 violations
├── Performance: 3 violations
├── Architecture: 3 violations
├── Implementation: 1 violation
└── PMAT: 1 violation
```

Most violations are in documentation (missing module/enum docs) and KISS principle (large structs), which are lower priority than architecture violations.
╔════════════════════════════════════════════════════════════════════════════════╗
║                    VIOLATIONS ANALYSIS - QUICK REFERENCE                       ║
╚════════════════════════════════════════════════════════════════════════════════╝

ISSUE #1: PAT004 - BrowseResult Definition
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ✅ FALSE POSITIVE
File: mcb-domain/src/ports/browse.rs:27
Issue: Report claims BrowseResult exists at line 27
Reality: Line 27 contains "pub enum HighlightError {" - no BrowseResult type alias
Action: IGNORE - No fix needed

ISSUE #2: IMPL001 - walk_directory_boxed Empty Body
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ⚠️ LIKELY FALSE POSITIVE
File: mcb-server/src/handlers/browse_service.rs:71
Issue: Report claims method returns None
Reality: Method is fully implemented (lines 55-111), returns Result<FileNode>
Action: VERIFY - Check with validator author or suppress rule

ISSUE #3: IMPL006 - Empty Catch-All Pattern
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ✅ FALSE POSITIVE
File: mcb-server/src/handlers/highlight_service.rs:125
Issue: Report claims empty catch-all '_ => {}'
Reality: Line 125 has "_ => Err(HighlightError::UnsupportedLanguage(...))"
Action: IGNORE - No fix needed

ISSUE #4: PERF002 - Allocation in Loop
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: 🔴 NEEDS FIX
File: mcb-server/src/admin/browse_handlers.rs:221
Issue: Vec::new() allocated inside loop (error fallback path)
Code:  Err(_) => HighlightedCode::new(c.content.clone(), Vec::new(), ...)
Fix:   Move Vec::new() outside loop or create once
Effort: LOW

ISSUE #5: PERF001 - Clones in Loop (Related to #4)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: 🔴 NEEDS FIX
File: mcb-server/src/admin/browse_handlers.rs:220, 222
Issue: c.content.clone() and c.language.clone() in loop
Code:  Lines 220 & 222 clone strings in error fallback
Fix:   Consider using references or move allocation outside loop
Effort: LOW

ISSUE #6: CA005 - Mutable Method in Value Object
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: 🔴 NEEDS FIX
File: mcb-domain/src/value_objects/browse/tree.rs:52
Issue: Value object has mutable method add_child(&mut self)
Code:  pub fn add_child(&mut self, child: FileTreeNode) { ... }
Fix:   Remove add_child(), use with_child() builder pattern instead
Usage: browse_handlers.rs:304 - node.add_child(file_node)
Effort: LOW

ISSUE #7: ORG016 - Domain Layer Method Violation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ⚠️ NEEDS INVESTIGATION
File: mcb-domain/src/entities/vcs/vcs_repo.rs:32
Issue: Domain entity has concrete method id() (should be trait-only?)
Code:  pub fn id(&self) -> &RepositoryId { &self.id }
Also:  VcsBranch::id, VcsBranch::name, VcsBranch::head_commit (similar)
Fix:   Clarify architectural rule - are getter methods allowed?
Effort: MEDIUM (requires architectural decision)

ISSUE #8: QUAL004 - Large File (project_repository)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: ✅ ALREADY FIXED
File: mcb-providers/src/database/sqlite/project_repository/
Issue: Report claims 559 lines
Reality: Already split into modules:

-   mod.rs (132 lines)
-   decision.rs, dependency.rs, issue.rs, phase.rs, project.rs
Action: IGNORE - No fix needed

Result TYPE ALIASES VERIFICATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ mcb-domain/src/error.rs: Result<T> = std::Result::Result<T, Error>
✅ mcb-validate/src/lib.rs: Result<T> = std::Result::Result<T, ValidationError>
✅ mcb-ast-utils/src/error.rs: Result<T> = std::Result::Result<T, AstError>
✅ mcb-language-support/src/error.rs: Result<T> = std::Result::Result<T, LanguageError>
⚠️ mcb-providers/src/workflow/transitions.rs: Result<T> = std::Result::Result<T, String>
   (Should use domain Error type instead of String)

NO BrowseResult or HighlightResult type aliases found in codebase.

╔════════════════════════════════════════════════════════════════════════════════╗
║                            FIX PRIORITY MATRIX                                 ║
╚════════════════════════════════════════════════════════════════════════════════╝

HIGH PRIORITY (Architecture Violations)
  [1] CA005: Remove mutable add_child() from FileTreeNode
      Effort: LOW | Impact: HIGH | Files: 1

MEDIUM PRIORITY (Performance)
  [2] PERF002 + PERF001: Reduce allocations/clones in browse_handlers.rs
      Effort: LOW | Impact: MEDIUM | Files: 1

LOW PRIORITY (Clarification)
  [3] ORG016: Clarify domain entity getter method rules
      Effort: MEDIUM | Impact: LOW | Files: 2+

FALSE POSITIVES (No Action)
  [✓] PAT004, IMPL001, IMPL006, QUAL004

VALIDATION REPORT SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total Violations: 329
├── Errors: 1 (PMAT005 - dead code analysis timeout)
├── Warnings: 204
└── Infos: 124

Top Categories:
├── Documentation: 108 (missing module/enum docs)
├── KISS: 79 (large structs)
├── Error Boundary: 50 (missing error context)
├── SOLID: 25 (SRP violations)
├── Organization: 26 (ORG016 getter methods)
└── Others: 41
