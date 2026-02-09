# Refactoring Plan

**Total smells:** 378

## HIGH (139 items)

### similar-code: `crates/mcb-validate/src/async_patterns.rs` L653–774

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 70.0
-   **Message:** Found 122 lines of similar code in 4 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/documentation.rs` L450–571

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 70.0
-   **Message:** Found 122 lines of similar code in 4 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/pattern_validator.rs` L610–718

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 70.0
-   **Message:** Found 109 lines of similar code in 4 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/performance.rs` L740–859

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 70.0
-   **Message:** Found 120 lines of similar code in 4 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/language/go.rs` L1–78

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 63.4
-   **Message:** Found 78 lines of similar code in 2 locations (mass = 215)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/language/kotlin.rs` L1–78

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 63.4
-   **Message:** Found 78 lines of similar code in 2 locations (mass = 215)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/schema/memory.rs` L197–254

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 57.4
-   **Message:** Found 58 lines of similar code in 2 locations (mass = 127)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/schema/project.rs` L193–250

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 57.4
-   **Message:** Found 58 lines of similar code in 2 locations (mass = 127)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/unit/mcp_protocol_tests.rs` L272–325

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 56.2
-   **Message:** Found 54 lines of similar code in 2 locations (mass = 309)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/unit/mcp_protocol_tests.rs` L329–382

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 56.2
-   **Message:** Found 54 lines of similar code in 2 locations (mass = 309)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/performance.rs` L503–552

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 55.0
-   **Message:** Found 50 lines of similar code in 3 locations (mass = 240)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/performance.rs` L589–638

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 55.0
-   **Message:** Found 50 lines of similar code in 3 locations (mass = 240)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/performance.rs` L670–719

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 55.0
-   **Message:** Found 50 lines of similar code in 3 locations (mass = 240)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L180–228

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 54.7
-   **Message:** Found 49 lines of similar code in 2 locations (mass = 293)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L310–357

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 54.4
-   **Message:** Found 48 lines of similar code in 2 locations (mass = 293)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/schema/project.rs` L61–105

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 53.5
-   **Message:** Found 45 lines of similar code in 2 locations (mass = 99)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/schema/project.rs` L107–151

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 53.5
-   **Message:** Found 45 lines of similar code in 2 locations (mass = 99)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L406–445

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 52.0
-   **Message:** Found 40 lines of similar code in 4 locations (mass = 125)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/anthropic.rs` L139–177

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.7
-   **Message:** Found 39 lines of similar code in 2 locations (mass = 189)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/openai.rs` L140–178

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.7
-   **Message:** Found 39 lines of similar code in 2 locations (mass = 189)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L486–524

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.7
-   **Message:** Found 39 lines of similar code in 4 locations (mass = 125)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/database/memory_provider.rs` L32–69

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.4
-   **Message:** Found 38 lines of similar code in 2 locations (mass = 215)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/ddl.rs` L80–117

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.4
-   **Message:** Found 38 lines of similar code in 2 locations (mass = 215)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/golden_tools_e2e_integration.rs` L207–243

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.1
-   **Message:** Found 37 lines of similar code in 2 locations (mass = 173)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/golden_tools_e2e_integration.rs` L268–304

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.1
-   **Message:** Found 37 lines of similar code in 2 locations (mass = 173)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L367–403

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 51.1
-   **Message:** Found 37 lines of similar code in 4 locations (mass = 125)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/admin/lifecycle_handlers.rs` L118–153

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 3 locations (mass = 160)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/admin/lifecycle_handlers.rs` L163–198

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 3 locations (mass = 160)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/admin/lifecycle_handlers.rs` L208–243

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 3 locations (mass = 160)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/project.rs` L470–505

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 2 locations (mass = 136)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/project.rs` L508–543

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 2 locations (mass = 136)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L448–483

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 50.8
-   **Message:** Found 36 lines of similar code in 4 locations (mass = 125)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L440–469

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 49.0
-   **Message:** Found 30 lines of similar code in 3 locations (mass = 126)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/memory_tests.rs` L229–257

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 48.7
-   **Message:** Found 29 lines of similar code in 2 locations (mass = 136)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/memory_tests.rs` L260–288

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 48.7
-   **Message:** Found 29 lines of similar code in 2 locations (mass = 136)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L368–392

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 2 locations (mass = 143)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L645–669

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 2 locations (mass = 143)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/index_codebase_tests.rs` L11–35

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 2 locations (mass = 135)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/index_codebase_tests.rs` L145–169

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 2 locations (mass = 135)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L413–437

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 3 locations (mass = 126)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L527–551

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 47.5
-   **Message:** Found 25 lines of similar code in 3 locations (mass = 126)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/vcs_tests.rs` L10–32

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.9
-   **Message:** Found 23 lines of similar code in 2 locations (mass = 102)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/vcs_tests.rs` L35–57

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.9
-   **Message:** Found 23 lines of similar code in 2 locations (mass = 102)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/value_objects/browse/highlight.rs` L5–26

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/clear_index_tests.rs` L34–55

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 96)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/get_indexing_status_tests.rs` L10–31

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 96)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/generic_reporter.rs` L220–241

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 88)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/generic_reporter.rs` L242–263

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 88)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/kiss.rs` L889–910

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/lib.rs` L365–386

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/solid/validator.rs` L472–493

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.6
-   **Message:** Found 22 lines of similar code in 2 locations (mass = 113)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/infrastructure/prometheus_metrics.rs` L209–229

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.3
-   **Message:** Found 21 lines of similar code in 2 locations (mass = 67)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/browse_api_integration.rs` L108–128

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.3
-   **Message:** Found 21 lines of similar code in 2 locations (mass = 67)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/modules/domain_services.rs` L44–63

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/provider_resolvers.rs` L65–84

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 90)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/provider_resolvers.rs` L143–162

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 90)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/mcp_server.rs` L43–62

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/session_tests.rs` L91–110

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 4 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/session_tests.rs` L113–132

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 4 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/session_tests.rs` L135–154

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 4 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/session_tests.rs` L157–176

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 4 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/pattern_validator.rs` L422–441

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 95)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/tests_org.rs` L633–652

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 46.0
-   **Message:** Found 20 lines of similar code in 2 locations (mass = 95)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L347–365

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 97)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L624–642

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 97)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/admin/auth_integration_tests.rs` L220–238

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 65)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/admin/auth_integration_tests.rs` L515–533

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 65)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/admin/auth_integration_tests.rs` L630–648

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 60)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/admin/auth_integration_tests.rs` L652–670

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 60)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/validate_tests.rs` L26–44

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 107)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/validate_tests.rs` L88–106

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 107)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/validate_tests.rs` L109–127

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 3 locations (mass = 102)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/validate_tests.rs` L130–148

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 3 locations (mass = 102)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/handlers/validate_tests.rs` L212–230

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 3 locations (mass = 102)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/implementation/validator.rs` L107–125

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 107)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/implementation/validator.rs` L190–208

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.7
-   **Message:** Found 19 lines of similar code in 2 locations (mass = 107)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L70–87

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 4 locations (mass = 80)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L89–106

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 4 locations (mass = 80)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L125–142

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 4 locations (mass = 80)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/infrastructure/lifecycle.rs` L151–168

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 3 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/infrastructure/lifecycle.rs` L171–188

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 3 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/infrastructure/lifecycle.rs` L191–208

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 3 locations (mass = 103)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-language-support/src/parser.rs` L31–48

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 60)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-language-support/src/parser.rs` L68–85

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 60)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/anthropic.rs` L66–83

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 4 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/openai.rs` L64–81

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 4 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/vector_store/milvus.rs` L365–382

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 92)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/extractor/rust_extractor.rs` L97–114

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 109)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/extractor/rust_extractor.rs` L118–135

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 109)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/kiss.rs` L786–803

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 98)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/kiss.rs` L843–860

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.4
-   **Message:** Found 18 lines of similar code in 2 locations (mass = 98)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L246–262

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 73)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L278–294

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 73)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-language-support/tests/unit/parser_tests.rs` L70–86

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 57)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/agent_repository.rs` L89–105

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/agent_repository.rs` L326–342

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/memory_repository.rs` L292–308

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 95)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L216–232

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 3 locations (mass = 97)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L312–328

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 3 locations (mass = 97)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L482–498

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 3 locations (mass = 97)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L66–82

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L84–100

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L102–118

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L180–196

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L271–287

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/project_repository.rs` L464–480

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/gemini.rs` L60–76

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 4 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/embedding/voyageai.rs` L64–80

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 4 locations (mass = 54)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/vector_store/edgevec.rs` L382–398

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 78)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/vector_store/edgevec.rs` L415–431

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 78)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/memory/session.rs` L70–86

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 98)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/session/summarize.rs` L43–59

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 45.1
-   **Message:** Found 17 lines of similar code in 2 locations (mass = 98)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-domain/src/ports/services/validation.rs` L26–41

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-infrastructure/src/di/admin.rs` L108–123

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 4 locations (mass = 80)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-language-support/tests/unit/parser_tests.rs` L52–67

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 57)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/memory_repository.rs` L85–100

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 95)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/database/sqlite/memory_repository.rs` L102–117

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 9 locations (mass = 91)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/src/vector_store/milvus.rs` L127–142

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 92)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/engines/hybrid_engine.rs` L347–362

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/engines/hybrid_engine.rs` L365–380

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/generic_reporter.rs` L47–62

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/metrics/thresholds.rs` L133–148

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 4 locations (mass = 116)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/metrics/thresholds.rs` L151–166

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 4 locations (mass = 116)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/metrics/thresholds.rs` L169–184

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 4 locations (mass = 116)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L151–166

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 55)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `scripts/fix_smells.py` L566–581

-   **Function:** `-`
-   **Language:** python
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 70)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `scripts/fix_smells.py` L765–780

-   **Function:** `-`
-   **Language:** python
-   **Score:** 44.8
-   **Message:** Found 16 lines of similar code in 2 locations (mass = 70)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-language-support/src/parser.rs` L175–189

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 2 locations (mass = 94)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L112–126

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 3 locations (mass = 70)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L160–174

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 3 locations (mass = 70)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-providers/tests/unit/project_repository_tests.rs` L177–191

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 3 locations (mass = 70)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/memory/handler.rs` L51–65

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 2 locations (mass = 101)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/src/handlers/memory/handler.rs` L67–81

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 2 locations (mass = 101)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/golden_memory_project_e2e.rs` L214–228

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 3 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/metrics/thresholds.rs` L187–201

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.5
-   **Message:** Found 15 lines of similar code in 4 locations (mass = 116)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/metrics/rca_analyzer.rs` L158–171

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.2
-   **Message:** Found 14 lines of similar code in 2 locations (mass = 94)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-validate/src/rules/registry.rs` L167–180

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 44.2
-   **Message:** Found 14 lines of similar code in 2 locations (mass = 55)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/golden_memory_project_e2e.rs` L260–271

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 43.6
-   **Message:** Found 12 lines of similar code in 3 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

### similar-code: `crates/mcb-server/tests/integration/golden_memory_project_e2e.rs` L278–289

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 43.6
-   **Message:** Found 12 lines of similar code in 3 locations (mass = 62)

**Strategy:** Deduplicate similar code blocks

Identify the varying parts between the similar blocks; extract the
common logic into a single helper parameterised by those differences.
  • Repeated traversal/loop boilerplate → a generic visitor or
    iterator helper accepting a callback or closure.
  • Near-identical branches → unify with a configuration parameter
    or data-driven lookup table.
  • Similar validation functions → table-driven approach iterating
    (predicate, handler) pairs.
  • Multiple functions differing only in type → use generics /
    templates / type parameters.

## MEDIUM (209 items)

### file-complexity: `crates/mcb-validate/src/organization/validator.rs` L1–915

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 196.0
-   **Message:** High total complexity (count = 272)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/tests_org.rs` L1–874

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 194.0
-   **Message:** High total complexity (count = 268)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/performance.rs` L1–860

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 188.5
-   **Message:** High total complexity (count = 257)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/kiss.rs` L1–927

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 183.5
-   **Message:** High total complexity (count = 247)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/solid/validator.rs` L1–717

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 180.5
-   **Message:** High total complexity (count = 241)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/naming.rs` L1–859

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 158.5
-   **Message:** High total complexity (count = 197)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L1–759

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 157.5
-   **Message:** High total complexity (count = 195)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/async_patterns.rs` L1–775

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 154.0
-   **Message:** High total complexity (count = 188)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/pattern_validator.rs` L1–719

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 150.5
-   **Message:** High total complexity (count = 181)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/quality.rs` L1–725

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 144.5
-   **Message:** High total complexity (count = 169)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-providers/src/vector_store/milvus.rs` L1–1039

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 132.5
-   **Message:** High total complexity (count = 145)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/refactoring.rs` L1–841

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 132.5
-   **Message:** High total complexity (count = 145)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/implementation/validator.rs` L1–800

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 130.0
-   **Message:** High total complexity (count = 140)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `scripts/fix_smells.py` L1–1598

-   **Function:** `-`
-   **Language:** python
-   **Score:** 127.0
-   **Message:** High total complexity (count = 134)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/error_boundary.rs` L1–462

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 123.5
-   **Message:** High total complexity (count = 127)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/documentation.rs` L1–572

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 118.5
-   **Message:** High total complexity (count = 117)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/tests_org.rs` L683–861

-   **Function:** `validate_test_quality`
-   **Language:** Rust
-   **Score:** 112.0
-   **Message:** Function with high complexity (count = 104): validate_test_quality

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### file-complexity: `crates/mcb-validate/src/duplication/detector.rs` L1–457

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 108.5
-   **Message:** High total complexity (count = 97)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/dependency.rs` L1–578

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 106.5
-   **Message:** High total complexity (count = 93)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/engines/rusty_rules_engine.rs` L1–428

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 104.0
-   **Message:** High total complexity (count = 88)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/lib.rs` L1–1002

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 104.0
-   **Message:** High total complexity (count = 88)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/quality.rs` L456–575

-   **Function:** `validate_no_unwrap_expect`
-   **Language:** Rust
-   **Score:** 104.0
-   **Message:** Function with high complexity (count = 88): validate_no_unwrap_expect

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/performance.rs` L286–388

-   **Function:** `validate_clone_in_loops`
-   **Language:** Rust
-   **Score:** 103.5
-   **Message:** Function with high complexity (count = 87): validate_clone_in_loops

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/tests_org.rs` L570–679

-   **Function:** `validate_test_function_naming`
-   **Language:** Rust
-   **Score:** 102.0
-   **Message:** Function with high complexity (count = 84): validate_test_function_naming

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/pattern_validator.rs` L387–490

-   **Function:** `validate_async_traits`
-   **Language:** Rust
-   **Score:** 99.5
-   **Message:** Function with high complexity (count = 79): validate_async_traits

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### file-complexity: `crates/mcb-validate/src/layer_flow.rs` L1–361

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 98.5
-   **Message:** High total complexity (count = 77)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/port_adapter.rs` L1–385

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 95.5
-   **Message:** High total complexity (count = 71)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/documentation.rs` L233–350

-   **Function:** `validate_pub_item_docs`
-   **Language:** Rust
-   **Score:** 93.0
-   **Message:** Function with high complexity (count = 66): validate_pub_item_docs

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/duplication/detector.rs` L213–356

-   **Function:** `tokenize_source`
-   **Language:** Rust
-   **Score:** 92.5
-   **Message:** Function with high complexity (count = 65): tokenize_source

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### file-complexity: `crates/mcb-validate/src/visibility.rs` L1–334

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 92.5
-   **Message:** High total complexity (count = 65)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/pattern_validator.rs` L281–384

-   **Function:** `validate_trait_based_di`
-   **Language:** Rust
-   **Score:** 88.5
-   **Message:** Function with high complexity (count = 57): validate_trait_based_di

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### file-complexity: `crates/mcb-validate/src/pmat.rs` L1–610

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 88.5
-   **Message:** High total complexity (count = 57)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L363–517

-   **Function:** `validate_trait_placement`
-   **Language:** Rust
-   **Score:** 88.0
-   **Message:** Function with high complexity (count = 56): validate_trait_placement

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/refactoring.rs` L517–629

-   **Function:** `validate_missing_test_files`
-   **Language:** Rust
-   **Score:** 87.0
-   **Message:** Function with high complexity (count = 54): validate_missing_test_files

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L45–176

-   **Function:** `validate_magic_numbers`
-   **Language:** Rust
-   **Score:** 86.5
-   **Message:** Function with high complexity (count = 53): validate_magic_numbers

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L181–269

-   **Function:** `validate_entity_identity`
-   **Language:** Rust
-   **Score:** 86.2
-   **Message:** Function with high complexity (count = 59): validate_entity_identity

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### file-complexity: `crates/mcb-validate/src/graph/dep_graph.rs` L1–165

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 85.5
-   **Message:** High total complexity (count = 51)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-providers/src/vector_store/edgevec.rs` L1–918

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 85.0
-   **Message:** High total complexity (count = 50)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### file-complexity: `crates/mcb-validate/src/metrics/thresholds.rs` L1–345

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 85.0
-   **Message:** High total complexity (count = 50)

**Strategy:** Split complex file into modules

Split the file into focused sub-modules:
  • Move helper/utility functions into a dedicated helpers module.
  • Separate each concern (e.g. parsing, validation,
    formatting) into its own module.
  • Extract types/data structures into a shared types module.
  • Re-export public items from the parent module index.
  • Keep each file focused on a single cohesive purpose.

### function-complexity: `crates/mcb-validate/src/async_patterns.rs` L235–363

-   **Function:** `validate_blocking_in_async`
-   **Language:** Rust
-   **Score:** 84.5
-   **Message:** Function with high complexity (count = 49): validate_blocking_in_async

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/kiss.rs` L684–776

-   **Function:** `validate_function_length`
-   **Language:** Rust
-   **Score:** 84.4
-   **Message:** Function with high complexity (count = 53): validate_function_length

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L631–761

-   **Function:** `validate_strict_directory`
-   **Language:** Rust
-   **Score:** 83.5
-   **Message:** Function with high complexity (count = 47): validate_strict_directory

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/performance.rs` L391–481

-   **Function:** `validate_allocation_in_loops`
-   **Language:** Rust
-   **Score:** 83.3
-   **Message:** Function with high complexity (count = 52): validate_allocation_in_loops

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/naming.rs` L644–745

-   **Function:** `validate_ca_naming`
-   **Language:** Rust
-   **Score:** 82.5
-   **Message:** Function with high complexity (count = 45): validate_ca_naming

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/async_patterns.rs` L546–643

-   **Function:** `validate_spawn_patterns`
-   **Language:** Rust
-   **Score:** 82.4
-   **Message:** Function with high complexity (count = 46): validate_spawn_patterns

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/async_patterns.rs` L366–451

-   **Function:** `validate_block_on_usage`
-   **Language:** Rust
-   **Score:** 80.3
-   **Message:** Function with high complexity (count = 49): validate_block_on_usage

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/vector_store/milvus.rs` L662–796

-   **Function:** `list_vectors`
-   **Language:** Rust
-   **Score:** 79.5
-   **Message:** Function with high complexity (count = 39): list_vectors

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/kiss.rs` L459–548

-   **Function:** `validate_function_params`
-   **Language:** Rust
-   **Score:** 79.5
-   **Message:** Function with high complexity (count = 45): validate_function_params

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L231–307

-   **Function:** `validate_lsp`
-   **Language:** Rust
-   **Score:** 79.1
-   **Message:** Function with high complexity (count = 52): validate_lsp

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L770–907

-   **Function:** `validate_domain_traits_only`
-   **Language:** Rust
-   **Score:** 78.5
-   **Message:** Function with high complexity (count = 37): validate_domain_traits_only

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/error_boundary.rs` L284–369

-   **Function:** `validate_layer_error_types`
-   **Language:** Rust
-   **Score:** 77.3
-   **Message:** Function with high complexity (count = 43): validate_layer_error_types

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L179–284

-   **Function:** `validate_duplicate_strings`
-   **Language:** Rust
-   **Score:** 77.0
-   **Message:** Function with high complexity (count = 34): validate_duplicate_strings

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/refactoring.rs` L352–463

-   **Function:** `validate_duplicate_definitions`
-   **Language:** Rust
-   **Score:** 77.0
-   **Message:** Function with high complexity (count = 34): validate_duplicate_definitions

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/tests_org.rs` L453–567

-   **Function:** `validate_test_naming`
-   **Language:** Rust
-   **Score:** 77.0
-   **Message:** Function with high complexity (count = 34): validate_test_naming

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/git/submodule.rs` L61–194

-   **Function:** `collect_submodules_sync`
-   **Language:** Rust
-   **Score:** 76.5
-   **Message:** Function with high complexity (count = 33): collect_submodules_sync

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/kiss.rs` L597–681

-   **Function:** `validate_nesting_depth`
-   **Language:** Rust
-   **Score:** 76.5
-   **Message:** Function with high complexity (count = 42): validate_nesting_depth

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/async_patterns.rs` L454–543

-   **Function:** `validate_mutex_types`
-   **Language:** Rust
-   **Score:** 76.0
-   **Message:** Function with high complexity (count = 38): validate_Mutex_types

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/organization/validator.rs` L524–623

-   **Function:** `validate_layer_violations`
-   **Language:** Rust
-   **Score:** 76.0
-   **Message:** Function with high complexity (count = 32): validate_layer_violations

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/pattern_validator.rs` L493–582

-   **Function:** `validate_result_types`
-   **Language:** Rust
-   **Score:** 75.0
-   **Message:** Function with high complexity (count = 36): validate_Result_types

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-server/src/handlers/validate.rs` L29–153

-   **Function:** `handle`
-   **Language:** Rust
-   **Score:** 74.5
-   **Message:** Function with high complexity (count = 29): handle

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/error_boundary.rs` L372–454

-   **Function:** `validate_leaked_errors`
-   **Language:** Rust
-   **Score:** 74.4
-   **Message:** Function with high complexity (count = 39): validate_leaked_errors

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/kiss.rs` L375–456

-   **Function:** `validate_struct_fields`
-   **Language:** Rust
-   **Score:** 74.1
-   **Message:** Function with high complexity (count = 39): validate_struct_fields

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/performance.rs` L558–641

-   **Function:** `validate_inefficient_iterators`
-   **Language:** Rust
-   **Score:** 73.7
-   **Message:** Function with high complexity (count = 37): validate_inefficient_iterators

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L525–627

-   **Function:** `validate_ca009_infrastructure_imports_application`
-   **Language:** Rust
-   **Score:** 73.5
-   **Message:** Function with high complexity (count = 27): validate_ca009_infrastructure_imports_application

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/error_boundary.rs` L206–281

-   **Function:** `validate_error_context`
-   **Language:** Rust
-   **Score:** 73.3
-   **Message:** Function with high complexity (count = 41): validate_error_context

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/metrics/thresholds.rs` L128–222

-   **Function:** `from_yaml`
-   **Language:** Rust
-   **Score:** 73.0
-   **Message:** Function with high complexity (count = 29): from_yaml

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/naming.rs` L331–404

-   **Function:** `validate_type_names`
-   **Language:** Rust
-   **Score:** 72.7
-   **Message:** Function with high complexity (count = 41): validate_type_names

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/naming.rs` L548–641

-   **Function:** `validate_file_suffixes`
-   **Language:** Rust
-   **Score:** 72.2
-   **Message:** Function with high complexity (count = 28): validate_file_suffixes

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/performance.rs` L644–722

-   **Function:** `validate_inefficient_strings`
-   **Language:** Rust
-   **Score:** 72.2
-   **Message:** Function with high complexity (count = 37): validate_inefficient_strings

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-infrastructure/src/database/memory_provider.rs` L20–160

-   **Function:** `generate_ddl`
-   **Language:** Rust
-   **Score:** 72.0
-   **Message:** Function with high complexity (count = 24): generate_ddl

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/pattern_registry/registry.rs` L59–161

-   **Function:** `load_rule_file`
-   **Language:** Rust
-   **Score:** 71.5
-   **Message:** Function with high complexity (count = 23): load_rule_file

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/git/project_detection/maven.rs` L28–113

-   **Function:** `parse_pom`
-   **Language:** Rust
-   **Score:** 71.3
-   **Message:** Function with high complexity (count = 31): parse_pom

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/performance.rs` L484–555

-   **Function:** `validate_arc_mutex_overuse`
-   **Language:** Rust
-   **Score:** 70.1
-   **Message:** Function with high complexity (count = 37): validate_Arc_Mutex_overuse

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L272–354

-   **Function:** `validate_value_object_immutability`
-   **Language:** Rust
-   **Score:** 69.9
-   **Message:** Function with high complexity (count = 30): validate_value_object_immutability

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/layer_flow.rs` L255–327

-   **Function:** `check_circular_dependencies`
-   **Language:** Rust
-   **Score:** 69.4
-   **Message:** Function with high complexity (count = 35): check_circular_dependencies

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/port_adapter.rs` L283–351

-   **Function:** `check_adapter_direct_usage`
-   **Language:** Rust
-   **Score:** 69.2
-   **Message:** Function with high complexity (count = 37): check_adapter_direct_usage

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/vector_store/milvus.rs` L896–998

-   **Function:** `get_chunks_by_file`
-   **Language:** Rust
-   **Score:** 69.0
-   **Message:** Function with high complexity (count = 18): get_chunks_by_file

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L360–443

-   **Function:** `validate_ca007_infrastructure_concrete_imports`
-   **Language:** Rust
-   **Score:** 68.7
-   **Message:** Function with high complexity (count = 27): validate_ca007_infrastructure_concrete_imports

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L57–133

-   **Function:** `validate_srp`
-   **Language:** Rust
-   **Score:** 67.6
-   **Message:** Function with high complexity (count = 29): validate_srp

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/dependency.rs` L268–329

-   **Function:** `validate_use_statements`
-   **Language:** Rust
-   **Score:** 66.1
-   **Message:** Function with high complexity (count = 35): validate_use_statements

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/port_adapter.rs` L209–281

-   **Function:** `check_port_trait_sizes`
-   **Language:** Rust
-   **Score:** 65.4
-   **Message:** Function with high complexity (count = 27): check_port_trait_sizes

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/tests_org.rs` L374–450

-   **Function:** `validate_test_directory_structure`
-   **Language:** Rust
-   **Score:** 65.1
-   **Message:** Function with high complexity (count = 24): validate_test_directory_structure

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/layer_flow.rs` L196–253

-   **Function:** `check_forbidden_imports`
-   **Language:** Rust
-   **Score:** 64.9
-   **Message:** Function with high complexity (count = 35): check_forbidden_imports

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/lib.rs` L532–605

-   **Function:** `validate_all`
-   **Language:** Rust
-   **Score:** 62.7
-   **Message:** Function with high complexity (count = 21): validate_all

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/visibility.rs` L241–297

-   **Function:** `check_utility_modules`
-   **Language:** Rust
-   **Score:** 61.6
-   **Message:** Function with high complexity (count = 29): check_utility_modules

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/git/project_detection/go.rs` L41–113

-   **Function:** `detect`
-   **Language:** Rust
-   **Score:** 61.4
-   **Message:** Function with high complexity (count = 19): detect

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-server/src/handlers/vcs/search_branch.rs` L13–78

-   **Function:** `search_branch`
-   **Language:** Rust
-   **Score:** 61.3
-   **Message:** Function with high complexity (count = 23): search_branch

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/engines/hybrid_engine.rs` L321–389

-   **Function:** `execute_lint_rule`
-   **Language:** Rust
-   **Score:** 61.2
-   **Message:** Function with high complexity (count = 21): execute_lint_rule

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `fix_cache.py` L7–60

-   **Function:** `add_save_if`
-   **Language:** python
-   **Score:** 61.2
-   **Message:** Function with high complexity (count = 30): add_save_if

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/engines/rusty_rules_engine.rs` L200–241

-   **Function:** `has_forbidden_dependency`
-   **Language:** Rust
-   **Score:** 61.1
-   **Message:** Function with high complexity (count = 37): has_forbidden_dependency

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/language/common/traverser.rs` L58–112

-   **Function:** `traverse_and_extract`
-   **Language:** Rust
-   **Score:** 61.0
-   **Message:** Function with high complexity (count = 29): traverse_and_extract

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-providers/src/vector_store/milvus.rs` L235–304

-   **Function:** `convert_search_results`
-   **Language:** Rust
-   **Score:** 60.5
-   **Message:** Function with high complexity (count = 19): convert_search_results

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L449–512

-   **Function:** `validate_ca008_application_port_imports`
-   **Language:** Rust
-   **Score:** 60.2
-   **Message:** Function with high complexity (count = 22): validate_ca008_application_port_imports

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/linters/parsers.rs` L99–160

-   **Function:** `parse_clippy_output`
-   **Language:** Rust
-   **Score:** 60.1
-   **Message:** Function with high complexity (count = 23): parse_clippy_output

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### nested-control-flow: `crates/mcb-validate/src/tests_org.rs` L748–853

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 60.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### function-complexity: `crates/mcb-validate/src/generic_reporter.rs` L144–212

-   **Function:** `to_human_readable`
-   **Language:** Rust
-   **Score:** 59.7
-   **Message:** Function with high complexity (count = 18): to_human_readable

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/naming.rs` L453–506

-   **Function:** `validate_constant_names`
-   **Language:** Rust
-   **Score:** 59.7
-   **Message:** Function with high complexity (count = 27): validate_constant_names

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/visibility.rs` L191–239

-   **Function:** `check_internal_helpers`
-   **Language:** Rust
-   **Score:** 58.7
-   **Message:** Function with high complexity (count = 28): check_internal_helpers

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `scripts/fix_smells.py` L1113–1166

-   **Function:** `report_plan`
-   **Language:** python
-   **Score:** 57.7
-   **Message:** Function with high complexity (count = 23): report_plan

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/metrics/thresholds.rs` L225–286

-   **Function:** `from_metrics_config`
-   **Language:** Rust
-   **Score:** 57.6
-   **Message:** Function with high complexity (count = 18): from_metrics_config

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/pmat.rs` L466–523

-   **Function:** `validate_tdg`
-   **Language:** Rust
-   **Score:** 56.9
-   **Message:** Function with high complexity (count = 19): validate_tdg

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/implementation/validator.rs` L129–175

-   **Function:** `validate_hardcoded_returns`
-   **Language:** Rust
-   **Score:** 56.6
-   **Message:** Function with high complexity (count = 25): validate_hardcoded_returns

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/quality.rs` L578–624

-   **Function:** `validate_no_panic`
-   **Language:** Rust
-   **Score:** 56.6
-   **Message:** Function with high complexity (count = 25): validate_no_panic

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/graph/dep_graph.rs` L43–88

-   **Function:** `build`
-   **Language:** Rust
-   **Score:** 55.3
-   **Message:** Function with high complexity (count = 23): build

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/naming.rs` L407–450

-   **Function:** `validate_function_names`
-   **Language:** Rust
-   **Score:** 55.2
-   **Message:** Function with high complexity (count = 24): validate_function_names

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/refactoring.rs` L632–683

-   **Function:** `validate_mod_declarations`
-   **Language:** Rust
-   **Score:** 55.1
-   **Message:** Function with high complexity (count = 19): validate_mod_declarations

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/duplication/detector.rs` L133–180

-   **Function:** `deduplicate_candidates`
-   **Language:** Rust
-   **Score:** 54.9
-   **Message:** Function with high complexity (count = 21): deduplicate_candidates

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/graph/dep_graph.rs` L116–163

-   **Function:** `check_layer_violation`
-   **Language:** Rust
-   **Score:** 54.9
-   **Message:** Function with high complexity (count = 21): check_layer_violation

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L360–410

-   **Function:** `validate_string_dispatch`
-   **Language:** Rust
-   **Score:** 54.3
-   **Message:** Function with high complexity (count = 18): validate_String_dispatch

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `fix_checkout.py` L7–57

-   **Function:** `add_persist_credentials`
-   **Language:** python
-   **Score:** 54.3
-   **Message:** Function with high complexity (count = 18): add_persist_credentials

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L180–228

-   **Function:** `validate_isp`
-   **Language:** Rust
-   **Score:** 53.7
-   **Message:** Function with high complexity (count = 18): validate_isp

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/clean_architecture/validator.rs` L131–178

-   **Function:** `validate_handler_injection`
-   **Language:** Rust
-   **Score:** 53.4
-   **Message:** Function with high complexity (count = 18): validate_handler_injection

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L310–357

-   **Function:** `validate_impl_method_count`
-   **Language:** Rust
-   **Score:** 53.4
-   **Message:** Function with high complexity (count = 18): validate_impl_method_count

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/tests_org.rs` L327–371

-   **Function:** `validate_no_inline_tests`
-   **Language:** Rust
-   **Score:** 52.5
-   **Message:** Function with high complexity (count = 18): validate_no_inline_tests

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/kiss.rs` L551–594

-   **Function:** `validate_builder_complexity`
-   **Language:** Rust
-   **Score:** 52.2
-   **Message:** Function with high complexity (count = 18): validate_builder_complexity

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/dependency.rs` L225–265

-   **Function:** `validate_cargo_dependencies`
-   **Language:** Rust
-   **Score:** 51.8
-   **Message:** Function with high complexity (count = 19): validate_Cargo_dependencies

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L136–177

-   **Function:** `validate_ocp`
-   **Language:** Rust
-   **Score:** 51.6
-   **Message:** Function with high complexity (count = 18): validate_ocp

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/duplication/fingerprint.rs` L178–215

-   **Function:** `find_duplicates`
-   **Language:** Rust
-   **Score:** 50.9
-   **Message:** Function with high complexity (count = 19): find_duplicates

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### function-complexity: `crates/mcb-validate/src/solid/validator.rs` L677–709

-   **Function:** `has_common_words`
-   **Language:** Rust
-   **Score:** 50.9
-   **Message:** Function with high complexity (count = 22): has_common_words

**Strategy:** Reduce function complexity

Break the function into smaller, focused helpers:
  • Extract nested loops into iterator or helper methods.
  • Replace deep conditional chains with early returns
    (guard clauses).
  • Move complex conditionals into named boolean variables.
  • Extract validation into dedicated predicate functions.
  • Use a table-driven pattern: define behaviours in a
    static lookup and iterate.
  • Each helper should do exactly one thing and be
    independently testable.

### nested-control-flow: `crates/mcb-validate/src/tests_org.rs` L612–671

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 48.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L443–483

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 42.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L267–300

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 40.2
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L347–377

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 39.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/documentation.rs` L297–323

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 38.1
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L585–610

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 37.8
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L355–377

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 36.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L422–441

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 36.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L236–253

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 35.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L644–659

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 34.8
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L622–636

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 34.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L491–505

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 34.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L97–109

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/engines/rusty_rules_engine.rs` L342–353

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.6
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L459–470

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.6
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L210–221

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.6
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L342–352

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L426–436

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L509–519

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L522–532

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L340–350

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L393–403

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L431–440

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/dependency.rs` L313–322

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L440–449

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L532–541

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L578–587

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L760–769

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/solid/validator.rs` L161–170

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 33.0
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L528–536

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/documentation.rs` L267–275

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/documentation.rs` L281–289

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/documentation.rs` L335–343

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/error_boundary.rs` L354–362

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/graph/dep_graph.rs` L148–156

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/layer_flow.rs` L239–247

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L361–369

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L375–383

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L389–397

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L435–443

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L477–485

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L491–499

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L540–548

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L626–634

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L707–715

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/port_adapter.rs` L337–345

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.7
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L222–229

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L255–262

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/clean_architecture/validator.rs` L332–339

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/dependency.rs` L248–255

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/error_boundary.rs` L440–447

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/port_adapter.rs` L266–273

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L540–547

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/refactoring.rs` L669–676

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.4
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L569–575

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.1
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/tests_org.rs` L358–364

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 32.1
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/duplication/detector.rs` L314–319

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.8
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/engines/rusty_rules_engine.rs` L229–234

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.8
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-providers/src/git/project_detection/maven.rs` L91–95

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/engines/rusty_rules_engine.rs` L219–223

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L431–435

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pmat.rs` L372–376

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pmat.rs` L493–497

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L492–496

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L502–506

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 31.5
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-server/src/handlers/vcs/search_branch.rs` L63–65

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L354–356

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/async_patterns.rs` L442–444

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/dependency.rs` L308–310

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/documentation.rs` L331–333

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/duplication/detector.rs` L305–307

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/duplication/fingerprint.rs` L200–202

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/error_boundary.rs` L263–265

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/graph/dep_graph.rs` L81–83

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L748–750

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/kiss.rs` L753–755

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/layer_flow.rs` L236–238

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/naming.rs` L431–433

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L330–332

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L335–337

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/pattern_validator.rs` L341–343

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L379–381

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/performance.rs` L472–474

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/quality.rs` L534–536

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/refactoring.rs` L550–552

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/refactoring.rs` L553–555

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `crates/mcb-validate/src/visibility.rs` L225–227

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 30.9
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `fix_cache.py` L35–36

-   **Function:** `-`
-   **Language:** python
-   **Score:** 30.6
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `scripts/fix_smells.py` L1158–1159

-   **Function:** `-`
-   **Language:** python
-   **Score:** 30.6
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

### nested-control-flow: `fix_cache.py` L41–41

-   **Function:** `-`
-   **Language:** python
-   **Score:** 30.3
-   **Message:** Deeply nested control flow (level = 5)

**Strategy:** Flatten deeply nested control flow

Reduce nesting depth to ≤4 levels:
  • Invert conditions and return/continue early
    (guard clauses).
  • Extract inner blocks into small, named helper functions.
  • Chain optional/nullable access with language combinators
    (``?.``, ``and_then``, ``map``, ``flatMap``).
  • Propagate errors with the language's idiomatic mechanism
    (``?``, ``try``, ``raise``/``throw``).
  • Consider the 'extract and compose' approach: each helper
    adds at most one level of nesting.

## LOW (30 items)

### return-statements: `crates/mcb-application/src/use_cases/memory_service.rs` L76–96

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 29.3
-   **Message:** Function with many returns (count = 6): matches_filter

**Strategy:** Reduce return statements

Too many return points make flow hard to follow:
  • Consolidate into a single exit by accumulating the
    Result in a variable.
  • Use guard clauses only at the top for preconditions.
  • Extract branches into small helper functions, each with
    a single return.
  • Consider replacing branching logic with a lookup table.

### function-parameters: `crates/mcb-application/src/use_cases/project_service.rs` L118–128

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 27.3
-   **Message:** Function with many parameters (count = 8): create_issue

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/handlers/project.rs` L401–411

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 27.3
-   **Message:** Function with many parameters (count = 8): create_issue

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `scripts/fix_smells.py` L1005–1014

-   **Function:** `-`
-   **Language:** python
-   **Score:** 27.0
-   **Message:** Function with many parameters (count = 8): filter_smells

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-application/src/use_cases/project_service.rs` L150–159

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 26.5
-   **Message:** Function with many parameters (count = 7): update_issue

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/handlers/project.rs` L414–423

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 26.5
-   **Message:** Function with many parameters (count = 7): update_issue

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-domain/src/entities/vcs/commit.rs` L75–83

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 26.2
-   **Message:** Function with many parameters (count = 7): new

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/admin/api.rs` L140–148

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 26.2
-   **Message:** Function with many parameters (count = 7): with_config_watcher

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `scripts/fix_smells.py` L383–391

-   **Function:** `-`
-   **Language:** python
-   **Score:** 26.2
-   **Message:** Function with many parameters (count = 7): run_qlty_smells

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-application/src/use_cases/project_service.rs` L250–258

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.7
-   **Message:** Function with many parameters (count = 6): create_decision

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/handlers/project.rs` L450–458

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.7
-   **Message:** Function with many parameters (count = 6): create_decision

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-validate/src/engines/expression_engine.rs` L148–156

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.7
-   **Message:** Function with many parameters (count = 6): execute_expression_rule

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-validate/src/engines/hybrid_engine.rs` L321–329

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.7
-   **Message:** Function with many parameters (count = 6): execute_lint_rule

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-domain/src/entities/workflow.rs` L166–173

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.4
-   **Message:** Function with many parameters (count = 6): new

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/init.rs` L316–323

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.4
-   **Message:** Function with many parameters (count = 6): run_HTTP_transport_with_admin

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### function-parameters: `crates/mcb-server/src/init.rs` L387–394

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.4
-   **Message:** Function with many parameters (count = 6): run_hybrid_transport_with_admin

**Strategy:** Reduce function parameter count

Group related parameters into a single configuration object:
  • Create a Config/Options type for 4+ parameters.
  • Use the builder pattern for complex construction.
  • Check if parameters can be derived from existing context.
  • Accept flexible input types to reduce overloads.
  • Default values should live in the config type, not the
    function signature.

### boolean-logic: `crates/mcb-validate/src/organization/validator.rs` L234–251

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 25.4
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/organization/validator.rs` L727–734

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 22.4
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/implementation/validator.rs` L602–608

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 22.1
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/implementation/validator.rs` L437–439

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.9
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/kiss.rs` L424–426

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.9
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/tests_org.rs` L821–823

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.9
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/tests_org.rs` L833–835

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.9
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/implementation/validator.rs` L589–590

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/naming.rs` L608–609

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/organization/validator.rs` L319–320

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/organization/validator.rs` L700–701

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/quality.rs` L541–542

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/quality.rs` L636–637

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.

### boolean-logic: `crates/mcb-validate/src/solid/validator.rs` L560–561

-   **Function:** `-`
-   **Language:** Rust
-   **Score:** 20.6
-   **Message:** Complex binary expression

**Strategy:** Simplify boolean expressions

Break complex boolean expressions into named predicates:
  • Extract ``a && b || c && d`` into a named variable
    (``is_valid = …``).
  • Use helper functions: ``is_test_file(path)``.
  • Apply De Morgan’s laws to simplify negations.
  • Use pattern-matching or match/when constructs if
    the language supports them.
