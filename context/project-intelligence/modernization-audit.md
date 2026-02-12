# Rust Modernization Audit: Authoritative Tools & Methods (2026)

**Last Updated**: February 12, 2026  
**Purpose**: Curated recommendations for Rust modernization covering dead code detection, dependency pruning, architecture simplification, and migration governance.

---

## 1. Dead Code Detection

### 1.1 Built-in Compiler Lint (`rustc`)
- **Official Tool**: Rust compiler's `dead_code` lint
- **Status**: ✅ Stable, built-in (Rust 1.0+)
- **Capabilities**:
  - Detects unused functions, methods, structs, enums, variables within a single crate
  - Enabled by default
  - Suppression: `#[allow(dead_code)]`
- **Limitations**: Single-crate analysis only (does not detect unused public APIs across workspace boundaries)
- **Adoption Cost**: Zero (built-in)
- **Migration Risk**: ⚠️ Low - May produce false positives for intentionally unused public APIs
- **Official Docs**: https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code

### 1.2 Clippy
- **Official Tool**: Rust community's standard static analysis tool
- **Status**: ✅ Stable, integrated with official Rust toolchain
- **Capabilities**:
  - Extensive lint catalog (400+ lints as of 2026)
  - Detects dead code, unused variables, and maintenance anti-patterns
  - Enforces idiomatic Rust practices
- **Installation**: `rustup component add clippy`
- **Usage**: `cargo clippy`
- **Adoption Cost**: Low (1-2 hours for initial setup + CI integration)
- **Migration Risk**: ⚠️ Low-Medium - May require code changes to satisfy lints
- **Official Docs**: https://doc.rust-lang.org/clippy/
- **GitHub**: https://github.com/rust-lang/rust-clippy

### 1.3 Warnalyzer (Multi-Crate Workspaces)
- **Tool**: Detects unused public APIs across crate boundaries
- **Status**: ⚠️ Active but niche (uses `scip` backend with `rust-analyzer` as of 2023)
- **Use Case**: Large workspaces where public items may be unused across crates
- **Adoption Cost**: Medium (requires setup, less mature than Clippy)
- **Migration Risk**: ⚠️ Medium - May flag intentionally public APIs for future use
- **GitHub**: https://github.com/est31/warnalyzer

**Recommendation**: Use `rustc` + `clippy` for 99% of projects. Consider `warnalyzer` only for large multi-crate workspaces with strict API hygiene requirements.

---

## 2. Dependency Pruning & Bloat Reduction

### 2.1 cargo-machete (Fast, Stable Rust)
- **Tool**: Fast unused dependency detection
- **Status**: ✅ Actively maintained (2026-01-25 update)
- **Capabilities**:
  - Scans source code for declared but unused crates
  - Works with stable Rust (no nightly required)
  - Automated fix: `cargo machete --fix`
  - Supports complex workspace configurations
  - Ignore patterns for false positives
- **Installation**: `cargo install cargo-machete`
- **Usage**: `cargo machete` (CI-friendly)
- **Adoption Cost**: Low (< 1 hour, drop-in CI integration)
- **Migration Risk**: ✅ Very Low - `--fix` automates removal, easy rollback
- **Official Docs**: https://crates.io/crates/cargo-machete
- **GitHub**: https://github.com/bnjbvr/cargo-machete

### 2.2 cargo-udeps (Accurate, Nightly Rust)
- **Tool**: Accurate unused dependency detection via compilation artifacts
- **Status**: ⚠️ Maintained but requires nightly Rust
- **Capabilities**:
  - Analyzes compiled artifacts for true dependency usage
  - More accurate than `cargo-machete` (fewer false positives)
- **Installation**: `cargo install cargo-udeps --locked`
- **Usage**: `cargo +nightly udeps`
- **Adoption Cost**: Medium (requires nightly toolchain, slower than machete)
- **Migration Risk**: ⚠️ Medium - Nightly dependency may break with Rust updates
- **Official Docs**: https://crates.io/crates/cargo-udeps
- **GitHub**: https://github.com/est31/cargo-udeps

### 2.3 cargo-bloat / cargo-bloated (Binary Size Analysis)
- **Tool**: Identifies which crates/functions contribute to binary size
- **Status**: ✅ `cargo-bloat` stable, `cargo-bloated` enhanced (May 2025)
- **Capabilities**:
  - Breakdown of space consumption by dependency
  - Helps identify heavy crates for replacement or feature disabling
  - `cargo-bloated`: Improved accuracy, strip-aware reporting (Linux/ELF)
- **Installation**: 
  - `cargo install cargo-bloat`
  - `cargo install cargo-bloated` (recommended for Linux)
- **Usage**: `cargo bloat --release --crates`
- **Adoption Cost**: Low (< 1 hour for analysis)
- **Migration Risk**: ✅ Very Low - Read-only analysis tool
- **Official Docs**: 
  - https://crates.io/crates/cargo-bloat
  - https://crates.io/crates/cargo-bloated
- **GitHub**: 
  - https://github.com/RazrFalcon/cargo-bloat
  - https://github.com/nnethercote/cargo-bloated

**Recommendation**: Use `cargo-machete` for CI/CD (fast, stable). Use `cargo-udeps` for thorough audits (accept nightly requirement). Use `cargo-bloated` for binary size optimization.

---

## 3. Dependency Governance & Security

### 3.1 cargo-deny (Comprehensive Governance)
- **Tool**: Lint dependency graph for advisories, bans, licenses, sources
- **Status**: ✅ Actively maintained (v0.18.10, Jan 8 2026)
- **Capabilities**:
  - **Advisories**: Detects security vulnerabilities via RustSec database
  - **Bans**: Deny/allow specific crates, detect duplicate versions, prevent wildcards
  - **Licenses**: Enforce SPDX license policies, detect copyleft/unlicensed crates
  - **Sources**: Ensure crates come from trusted sources (supply chain security)
  - **Workspace Dependencies**: Lint `[workspace.dependencies]` for unused deps
- **Installation**: `cargo install cargo-deny`
- **Usage**: `cargo deny check` (CI-friendly)
- **Configuration**: `deny.toml` (project root)
- **Adoption Cost**: Medium (2-4 hours for initial policy definition)
- **Migration Risk**: ⚠️ Medium - May require dependency replacements for license/security compliance
- **Official Docs**: https://embarkstudios.github.io/cargo-deny/
- **GitHub**: https://github.com/EmbarkStudios/cargo-deny

### 3.2 cargo-audit (Security Vulnerability Scanning)
- **Tool**: Audits `Cargo.lock` against RustSec Advisory Database
- **Status**: ✅ Maintained by Rust Secure Code Working Group (2026 advisories active)
- **Capabilities**:
  - Detects known security vulnerabilities
  - Identifies yanked crates
  - Tracks vulnerabilities in Rust core components
  - Experimental `cargo audit fix` for auto-updates
- **Installation**: `cargo install cargo-audit`
- **Usage**: `cargo audit`
- **Adoption Cost**: Low (< 1 hour, drop-in CI integration)
- **Migration Risk**: ⚠️ Low-Medium - May require dependency updates or version pins
- **Official Docs**: https://rustsec.org/
- **GitHub**: https://github.com/rustsec/rustsec

**Recommendation**: Use `cargo-deny` for comprehensive governance (licenses + security + bans). Use `cargo-audit` as a lightweight alternative if only security scanning is needed.

---

## 4. Compile-Time Architecture Guardrails

### 4.1 TangleGuard (Architecture Enforcement)
- **Tool**: Static analysis for architectural rule enforcement
- **Status**: ⚠️ Public beta (June 2025), actively developed
- **Capabilities**:
  - Define architectural layers and boundaries for Cargo workspaces
  - Scan for rule violations (e.g., prevent UI layer from calling DB directly)
  - Interactive graph visualization of module connections
  - Impact analysis for refactoring decisions
  - Templates for layered architecture, clean architecture
- **Installation**: CLI tool with web UI
- **Configuration**: JSON file in repository
- **Adoption Cost**: Medium-High (4-8 hours for rule definition + team training)
- **Migration Risk**: ⚠️ Medium - May reveal existing architectural violations requiring refactoring
- **Official Docs**: https://tangleguard.com/
- **Supported Languages**: Rust, JavaScript, PHP

### 4.2 cargo-modules (Internal Dependency Visualization)
- **Tool**: Visualize internal module dependencies within a crate
- **Status**: ✅ Stable
- **Capabilities**:
  - Print crate's internal dependencies as graph
  - Output in Graphviz DOT format
- **Installation**: `cargo install cargo-modules`
- **Usage**: `cargo modules dependencies | dot -Tpng > deps.png`
- **Adoption Cost**: Low (< 1 hour)
- **Migration Risk**: ✅ Very Low - Read-only visualization
- **Official Docs**: https://crates.io/crates/cargo-modules
- **GitHub**: https://github.com/regexident/cargo-modules

### 4.3 cargo-visualize (Interactive Crate Dependency Graph)
- **Tool**: Interactive dependency graph visualization (fork of `cargo-depgraph`)
- **Status**: ✅ Updated Nov 2025
- **Capabilities**:
  - Highlight different dependency types (normal, dev, build, optional)
  - Interactive graph exploration
- **Installation**: `cargo install cargo-visualize`
- **Adoption Cost**: Low (< 1 hour)
- **Migration Risk**: ✅ Very Low - Read-only visualization
- **GitHub**: https://github.com/jplatte/cargo-visualize

**Recommendation**: Use `TangleGuard` for enforcing architectural boundaries in large projects. Use `cargo-modules` + `cargo-visualize` for understanding existing structure.

---

## 5. API Deprecation Workflows

### 5.1 Rust `#[deprecated]` Attribute (Built-in)
- **Official Feature**: Compile-time deprecation warnings
- **Status**: ✅ Stable (Rust 1.0+)
- **Capabilities**:
  - Apply to functions, methods, traits, struct fields, enum variants
  - `since` field: Version when deprecated (semantic versioning)
  - `note` field: Human-readable reason + migration guidance
  - Compiler issues warnings on usage
  - `rustdoc` displays deprecation in generated docs
- **Syntax**:
  ```rust
  #[deprecated(since = "1.2.0", note = "Use `new_function` instead")]
  pub fn old_function() { }
  ```
- **Adoption Cost**: Zero (built-in)
- **Migration Risk**: ✅ Very Low - Non-breaking, warnings only
- **Official Docs**: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute

### 5.2 cargo-semver-checks (Breaking Change Detection)
- **Tool**: Detects API breaking changes for SemVer compliance
- **Status**: ✅ Actively maintained (242+ lints as of end 2025)
- **Capabilities**:
  - Analyzes `rustdoc` JSON output to compare public API against baseline
  - Detects: removed structs/enums, non-exhaustive changes, trait sealing, type changes
  - Reports specific item, file location, line number + SemVer rule reference
  - Zero false positives design goal
  - Used by `tokio`, `PyO3`, `cargo` itself
- **Installation**: `cargo install cargo-semver-checks`
- **Usage**: `cargo semver-checks`
- **Adoption Cost**: Low (< 2 hours for CI integration)
- **Migration Risk**: ⚠️ Low-Medium - May block releases until breaking changes are fixed or version bumped
- **Future (2026+)**: Witness generation for type-checking lints, integration with `cargo publish`
- **Official Docs**: https://crates.io/crates/cargo-semver-checks
- **GitHub**: https://github.com/obi1kenobi/cargo-semver-checks

**Recommendation**: Use `#[deprecated]` for all API deprecations. Use `cargo-semver-checks` in CI to prevent accidental breaking changes.

---

## 6. ADR Lifecycle Automation

### 6.1 adrs (joshrotenberg/adrs)
- **Tool**: CLI for creating and managing Architecture Decision Records
- **Status**: ✅ Actively maintained (Rust-based)
- **Capabilities**:
  - Compatible with existing `adr-tools` repositories
  - Supports Nygard (classic) and MADR 4.0.0 formats
  - Template variants, tag support, full-text search
  - Repository health checks
  - **MCP Server**: Integration with AI agents for automated ADR analysis
- **Installation**: 
  - Homebrew: `brew install joshrotenberg/tap/adrs`
  - Cargo: `cargo install adrs`
  - Docker: Available
- **Adoption Cost**: Low-Medium (2-4 hours for team onboarding)
- **Migration Risk**: ✅ Very Low - Compatible with existing ADR workflows
- **Official Docs**: https://joshrotenberg.com/adrs/
- **GitHub**: https://github.com/joshrotenberg/adrs

### 6.2 ADRust
- **Tool**: CLI for ADR lifecycle management (Rust-based)
- **Status**: ⚠️ Active but not on crates.io (requires `git clone`)
- **Capabilities**:
  - Create, obsolete, tag, search ADRs
  - Asciidoc templates
  - Search across titles, content, dates, tags
- **Installation**: `git clone` + `cargo build`
- **Adoption Cost**: Medium (requires manual build)
- **Migration Risk**: ⚠️ Medium - Not published on crates.io, less mature
- **GitHub**: https://github.com/unexist/ADRust

### 6.3 record-tools-rs
- **Tool**: Manages ADRs + Technical Debt Records
- **Status**: ⚠️ Active (Rust-based)
- **Capabilities**: Combined approach to architectural and technical debt documentation
- **Adoption Cost**: Medium
- **Migration Risk**: ⚠️ Medium - Less mature than `adrs`
- **GitHub**: https://github.com/unexist/record-tools-rs

**Recommendation**: Use `adrs` for ADR automation (mature, MCP integration, multi-format support). Consider `ADRust` only if Asciidoc is a hard requirement.

---

## 7. Migration Risk Matrix

| Tool | Adoption Cost | Migration Risk | Maintenance Burden | Recommended For |
|------|---------------|----------------|-------------------|-----------------|
| **rustc dead_code lint** | Zero | ✅ Very Low | Zero | All projects |
| **clippy** | Low | ⚠️ Low-Medium | Low | All projects |
| **cargo-machete** | Low | ✅ Very Low | Low | CI/CD pipelines |
| **cargo-udeps** | Medium | ⚠️ Medium | Medium | Thorough audits |
| **cargo-bloat/bloated** | Low | ✅ Very Low | Low | Binary size optimization |
| **cargo-deny** | Medium | ⚠️ Medium | Medium | Governance-heavy orgs |
| **cargo-audit** | Low | ⚠️ Low-Medium | Low | Security-focused teams |
| **TangleGuard** | Medium-High | ⚠️ Medium | Medium | Large architectural projects |
| **cargo-modules** | Low | ✅ Very Low | Low | Understanding structure |
| **cargo-visualize** | Low | ✅ Very Low | Low | Dependency exploration |
| **#[deprecated]** | Zero | ✅ Very Low | Zero | All public APIs |
| **cargo-semver-checks** | Low | ⚠️ Low-Medium | Low | Library authors |
| **adrs** | Low-Medium | ✅ Very Low | Low | Teams using ADRs |

---

## 8. Recommended Modernization Workflow

### Phase 1: Foundation (Week 1)
1. **Enable Clippy in CI**: `cargo clippy -- -D warnings`
2. **Add cargo-machete to CI**: Detect unused dependencies weekly
3. **Integrate cargo-audit**: Security scanning on every PR
4. **Adopt #[deprecated]**: Mark all deprecated APIs

### Phase 2: Governance (Week 2-3)
1. **Configure cargo-deny**: Define license policies, ban problematic crates
2. **Run cargo-semver-checks**: Prevent breaking changes in CI
3. **Visualize dependencies**: Use `cargo-visualize` to understand current state

### Phase 3: Architecture (Week 4-6)
1. **Define architectural rules**: Use TangleGuard for large projects
2. **Document decisions**: Set up `adrs` for ADR automation
3. **Analyze binary size**: Use `cargo-bloated` to identify optimization targets

### Phase 4: Continuous Improvement
1. **Monthly audits**: Run `cargo-udeps` for deep dependency analysis
2. **Quarterly reviews**: Update `cargo-deny` policies, review ADRs
3. **Annual refactoring**: Address architectural violations flagged by TangleGuard

---

## 9. Version Caveats & Compatibility

| Tool | Rust Version | Notes |
|------|--------------|-------|
| **cargo-udeps** | Nightly required | May break with Rust updates |
| **cargo-machete** | Stable 1.70+ | Recommended for CI |
| **cargo-deny** | Stable 1.70+ | MSRV policy enforced |
| **cargo-semver-checks** | Stable 1.74+ | Requires `rustdoc` JSON |
| **TangleGuard** | Stable 1.70+ | Beta software, expect changes |
| **adrs** | Stable 1.70+ | Mature, stable API |

---

## 10. Official Links & Resources

### Documentation
- **Rust Reference (Deprecation)**: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute
- **Clippy Lints**: https://rust-lang.github.io/rust-clippy/master/
- **RustSec Advisory Database**: https://rustsec.org/
- **cargo-deny Book**: https://embarkstudios.github.io/cargo-deny/

### GitHub Repositories
- **cargo-machete**: https://github.com/bnjbvr/cargo-machete
- **cargo-udeps**: https://github.com/est31/cargo-udeps
- **cargo-deny**: https://github.com/EmbarkStudios/cargo-deny
- **cargo-audit**: https://github.com/rustsec/rustsec
- **cargo-semver-checks**: https://github.com/obi1kenobi/cargo-semver-checks
- **adrs**: https://github.com/joshrotenberg/adrs
- **TangleGuard**: https://tangleguard.com/

### Crates.io
- **cargo-machete**: https://crates.io/crates/cargo-machete
- **cargo-udeps**: https://crates.io/crates/cargo-udeps
- **cargo-bloat**: https://crates.io/crates/cargo-bloat
- **cargo-bloated**: https://crates.io/crates/cargo-bloated
- **cargo-deny**: https://crates.io/crates/cargo-deny
- **cargo-audit**: https://crates.io/crates/cargo-audit
- **cargo-semver-checks**: https://crates.io/crates/cargo-semver-checks
- **adrs**: https://crates.io/crates/adrs

---

## 11. Exclusions & Non-Recommendations

**Excluded Tools** (Low quality, unmaintained, or unverifiable):
- ❌ `cargo-deadcode` (no widely recognized tool by this name)
- ❌ `cargo-geiger` (specialized for `unsafe` code, not general dead code detection)
- ❌ Blog-only tools without official repos or crates.io presence
- ❌ Tools without 2025-2026 maintenance activity

**Why Excluded**:
- No official documentation or GitHub presence
- Unmaintained (last commit > 2 years ago)
- Superseded by better alternatives (e.g., `cargo tree` vs. custom dep visualizers)
- Unverifiable claims or low-quality sources

---

## 12. Repository-Specific Findings (MCB)

### P0

- Multi-tenant org fallback in admin browse handlers:
  - `crates/mcb-server/src/admin/handlers.rs:180`
  - `crates/mcb-server/src/admin/handlers.rs:253`
  - `crates/mcb-server/src/admin/handlers.rs:298`
  - `crates/mcb-server/src/admin/handlers.rs:336`

Current behavior still uses `OrgContext::default()` in protected paths. Prioritize auth-derived org context extraction and fail-closed behavior when missing.

### P1

- Production dead-code suppression:
  - `crates/mcb-providers/src/database/sqlite/mod.rs:11`

Remove `#[allow(dead_code)]` and either remove or wire remaining code paths.

- ADR status inconsistency:
  - `docs/adr/030-multi-provider-strategy.md:4`
  - `docs/adr/030-multi-provider-strategy.md:10`

Reconcile `status: IMPLEMENTED` with `implementation_status: Incomplete` and define objective closure criteria.

- Superseded ADR lineage that still causes planning noise:
  - `docs/adr/012-di-strategy-two-layer-approach.md:9`
  - `docs/adr/024-simplified-dependency-injection.md:9`
  - `docs/adr/032-agent-quality-domain-extension.md:9`

### P2

- Duplicated handler test resolver/mock patterns:
  - `crates/mcb-server/src/handlers/vcs_entity.rs:239`
  - `crates/mcb-server/src/handlers/project.rs:88`
  - `crates/mcb-server/src/handlers/plan_entity.rs:176`
  - `crates/mcb-server/src/handlers/issue_entity.rs:228`
  - `crates/mcb-server/src/handlers/org_entity.rs:30`

- Commented-out tests without explicit restore/remove policy:
  - `crates/mcb-server/tests/integration.rs:54`
  - `crates/mcb-server/tests/integration.rs:83`
  - `crates/mcb-server/tests/unit.rs:24`
  - `crates/mcb-server/tests/unit.rs:33`
  - `crates/mcb-server/tests/unit.rs:38`

- CRUD adapter boilerplate concentration:
  - `crates/mcb-server/src/admin/crud_adapter.rs:26`
  - `crates/mcb-server/src/admin/crud_adapter.rs:55`
  - `crates/mcb-server/src/admin/crud_adapter.rs:246`

- Search abstraction overlap:
  - `crates/mcb-application/src/use_cases/search_service.rs:12`
  - `crates/mcb-domain/src/repositories/search_repository.rs:35`

- Parallel indexing orchestration paths:
  - `crates/mcb-application/src/use_cases/indexing_service.rs`
  - `crates/mcb-application/src/use_cases/vcs_indexing.rs:79`

### Validation note

- `todo!()`/`unimplemented!()` evidence at
  - `crates/mcb-validate/src/implementation/validator.rs:692`
  - `crates/mcb-validate/src/implementation/validator.rs:695`

These are test fixture strings in test code, not production runtime panic paths.

## 13. RMCP/Rocket Architecture Intake

From external RMCP/Rocket research, these patterns map directly to `mcb-server` modernization:

- Registry-based, typed tool dispatch and unified failure mapping in
  - `crates/mcb-server/src/tools/router.rs`
- Transport-agnostic core with explicit protocol version negotiation in
  - `crates/mcb-server/src/mcp_server.rs`
- Lifecycle hardening (startup/shutdown, in-flight drain)
- Bounded observability tags (`tool_name`, `call_id`, `duration_ms`, `success`)
- Cached schema metadata for frequently accessed tool definitions

**End of Modernization Audit**
**Maintained by**: MCB Project Intelligence
**Next Review**: Q2 2026
