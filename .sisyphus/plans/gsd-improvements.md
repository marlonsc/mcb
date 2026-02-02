# GSD Improvements Work Plan

## TL;DR

> **Quick Summary**: Improve the GSD workflow system by reducing complexity, consolidating documentation, and optimizing context usage. Focus on high-impact quick wins first.
> 
> **Deliverables**:
> - Refactored large files (execute-plan.md split into 5 modules)
> - New /gsd-start unified entry point command
> - Improved /gsd-help with visual categorization
> - Consolidated documentation with single source of truth per concept
> - Context-optimized @references
> 
> **Estimated Effort**: Large (~15-20 days total, split across 5 phases)
> **Parallel Execution**: YES - Phases can be partially parallelized
> **Critical Path**: Phase 1 (Quick Wins) → Phase 2 (Consolidation) → Phase 3 (Context)

---

## Context

### Original Request
User requested comprehensive analysis and improvement of the GSD (Get Shit Done) project to reduce complexity and improve usability.

### Interview Summary
**Key Discussions**:
- GSD is at v1.11.1 with 27 commands, 11 agents, ~14,000 lines
- Main pain points: steep learning curve, large files, concept redundancy
- User wants comprehensive improvement (UX + technical)

**Research Findings**:
- execute-plan.md: 1844 lines (largest file, hard to maintain)
- checkpoints.md: 1078 lines (duplicates content in execute-plan)
- Same concepts explained in 3+ places
- No automated tests for workflow validation

### Project Location
`~/.config/opencode/get-shit-done/`

---

## Work Objectives

### Core Objective
Transform GSD from a complex 27-command system into a more approachable, maintainable workflow tool while preserving its powerful capabilities.

### Concrete Deliverables
- `execute-plan-core.md` (~400 lines) - core execution flow
- `execute-plan-checkpoints.md` (~300 lines) - checkpoint handling
- `execute-plan-segments.md` (~300 lines) - segmentation logic
- `execute-plan-git.md` (~200 lines) - git/commit operations
- `execute-plan-deviations.md` (~300 lines) - deviation rules
- `gsd-start.md` - new unified entry point command
- Updated `gsd-help.md` - visual categorization
- `references/INDEX.md` - concept index
- Updated cross-references across all files

### Definition of Done
- [ ] No file exceeds 600 lines
- [ ] Each concept has single source of truth
- [ ] /gsd-start routes correctly based on project state
- [ ] /gsd-help shows simplified view by default
- [ ] All @references point to correct modular files

### Must Have
- Backward compatibility for existing .planning/ directories
- All current functionality preserved
- Clear migration path for users

### Must NOT Have (Guardrails)
- DO NOT remove any existing commands (deprecate instead)
- DO NOT change STATE.md format (breaking change)
- DO NOT modify PLAN.md or SUMMARY.md templates
- DO NOT add new dependencies
- DO NOT change the core workflow (new-project → plan → execute → verify)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (markdown-based system)
- **User wants tests**: Manual verification
- **Framework**: None (prompt-based system)

### Manual Verification Protocol

Each TODO includes verification steps that can be executed manually:

1. **File splitting verification**: `wc -l` on new files
2. **Command routing verification**: Run /gsd-start in different contexts
3. **Reference verification**: grep for old @references, ensure none remain

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── TODO 1: Split execute-plan.md into modules
├── TODO 2: Split checkpoints.md into modules
└── TODO 3: Create /gsd-start command

Wave 2 (After Wave 1):
├── TODO 4: Update all @references to point to new modules
├── TODO 5: Create references/INDEX.md
└── TODO 6: Improve gsd-help.md

Wave 3 (After Wave 2):
├── TODO 7: Consolidate duplicate content
└── TODO 8: Add deprecation notices

Wave 4 (Final):
└── TODO 9: Validation and documentation
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|------|------------|--------|---------------------|
| 1 | None | 4, 7 | 2, 3 |
| 2 | None | 4, 7 | 1, 3 |
| 3 | None | 6 | 1, 2 |
| 4 | 1, 2 | 7 | 5, 6 |
| 5 | None | 9 | 4, 6 |
| 6 | 3 | 9 | 4, 5 |
| 7 | 4 | 8, 9 | None |
| 8 | 7 | 9 | None |
| 9 | 5, 6, 8 | None | None |

---

## TODOs

### TODO 1: Split execute-plan.md into Modular Files

**What to do**:
- Read `~/.config/opencode/get-shit-done/workflows/execute-plan.md` (1844 lines)
- Identify logical sections:
  - Core execution flow (steps, segments, main loop)
  - Checkpoint handling (types, protocol, gates)
  - Segmentation logic (when to segment, how)
  - Git operations (commits, state updates)
  - Deviation handling (rules, tolerance)
- Create 5 new files in `workflows/`:
  - `execute-plan-core.md` - main execution steps
  - `execute-plan-checkpoints.md` - checkpoint protocol
  - `execute-plan-segments.md` - segmentation
  - `execute-plan-git.md` - git/commits
  - `execute-plan-deviations.md` - deviation rules
- Update original `execute-plan.md` to be a thin orchestrator that includes @references to modules

**Must NOT do**:
- Remove any content (only reorganize)
- Change any logic or behavior
- Modify template formats

**Recommended Agent Profile**:
- **Category**: `unspecified-high`
  - Reason: Large refactoring task requiring careful content preservation
- **Skills**: `[]`
  - No special skills needed - text manipulation only

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Tasks 2, 3)
- **Blocks**: Tasks 4, 7
- **Blocked By**: None (can start immediately)

**References**:
- `~/.config/opencode/get-shit-done/workflows/execute-plan.md` - Source file to split
- `~/.config/opencode/get-shit-done/workflows/execute-phase.md:1-50` - Example of modular structure

**Acceptance Criteria**:
- [ ] `wc -l workflows/execute-plan-core.md` returns < 500
- [ ] `wc -l workflows/execute-plan-checkpoints.md` returns < 400
- [ ] `wc -l workflows/execute-plan-segments.md` returns < 400
- [ ] `wc -l workflows/execute-plan-git.md` returns < 300
- [ ] `wc -l workflows/execute-plan-deviations.md` returns < 400
- [ ] Original execute-plan.md references all 5 modules
- [ ] `grep -c "@.*execute-plan-" workflows/execute-plan.md` returns 5

**Commit**: YES
- Message: `refactor(gsd): split execute-plan.md into 5 modular files`
- Files: `workflows/execute-plan*.md`

---

### TODO 2: Split checkpoints.md into Modular Files

**What to do**:
- Read `~/.config/opencode/get-shit-done/references/checkpoints.md` (1078 lines)
- Identify logical sections:
  - Checkpoint types (human-verify, decision, human-action)
  - Execution protocol (how to handle each type)
  - Automation reference (CLI commands, env vars)
- Create 3 new files in `references/`:
  - `checkpoints-types.md` - type definitions and when to use each
  - `checkpoints-protocol.md` - execution protocol
  - `checkpoints-automation.md` - automation reference
- Update original to be orchestrator with @references

**Must NOT do**:
- Change checkpoint type definitions
- Modify execution protocol
- Remove any examples

**Recommended Agent Profile**:
- **Category**: `unspecified-high`
  - Reason: Large refactoring requiring content preservation
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Tasks 1, 3)
- **Blocks**: Tasks 4, 7
- **Blocked By**: None

**References**:
- `~/.config/opencode/get-shit-done/references/checkpoints.md` - Source file to split

**Acceptance Criteria**:
- [ ] `wc -l references/checkpoints-types.md` returns < 400
- [ ] `wc -l references/checkpoints-protocol.md` returns < 400
- [ ] `wc -l references/checkpoints-automation.md` returns < 400
- [ ] Original checkpoints.md references all 3 modules
- [ ] All checkpoint examples preserved

**Commit**: YES
- Message: `refactor(gsd): split checkpoints.md into 3 modular files`
- Files: `references/checkpoints*.md`

---

### TODO 3: Create /gsd-start Unified Entry Point

**What to do**:
- Create new command file `~/.config/opencode/command/gsd-start.md`
- Implement context detection:
  - No .planning/ → suggest /gsd-new-project
  - .planning/ exists but no phases started → suggest /gsd-plan-phase 1
  - Mid-execution (incomplete SUMMARY) → suggest /gsd-resume-work
  - Phase complete → show progress and suggest next phase
- Display brief status summary
- Route to appropriate command

**Must NOT do**:
- Duplicate functionality of existing commands
- Make decisions for user (only suggest)
- Add complex logic

**Recommended Agent Profile**:
- **Category**: `quick`
  - Reason: Single file creation with clear spec
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Tasks 1, 2)
- **Blocks**: Task 6
- **Blocked By**: None

**References**:
- `~/.config/opencode/command/gsd-progress.md` - Similar routing logic
- `~/.config/opencode/command/gsd-resume-work.md` - Context detection pattern

**Acceptance Criteria**:
- [ ] File exists: `~/.config/opencode/command/gsd-start.md`
- [ ] Command detects no .planning/ and suggests new-project
- [ ] Command detects incomplete phase and suggests resume
- [ ] Command shows brief status when project exists
- [ ] File size < 200 lines

**Commit**: YES
- Message: `feat(gsd): add /gsd-start unified entry point`
- Files: `command/gsd-start.md`

---

### TODO 4: Update All @references to New Modules

**What to do**:
- Search all files for references to old monolithic files
- Update @references to point to appropriate modules:
  - `@execute-plan.md` → specific module or keep as orchestrator
  - `@checkpoints.md` → specific module based on context
- Ensure no broken references remain

**Must NOT do**:
- Change any behavior
- Remove necessary references
- Add unnecessary references

**Recommended Agent Profile**:
- **Category**: `unspecified-low`
  - Reason: Search and replace task
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2 (with Tasks 5, 6)
- **Blocks**: Task 7
- **Blocked By**: Tasks 1, 2

**References**:
- All files in `~/.config/opencode/get-shit-done/`

**Acceptance Criteria**:
- [ ] `grep -r "execute-plan.md" --include="*.md" | wc -l` returns count of valid references only
- [ ] No broken @references when loading agents
- [ ] All modules correctly referenced

**Commit**: YES
- Message: `refactor(gsd): update @references to modular files`
- Files: Multiple files with reference updates

---

### TODO 5: Create references/INDEX.md

**What to do**:
- Create comprehensive index of all GSD concepts
- Organize by category:
  - Workflow concepts (phases, plans, execution)
  - Checkpoint concepts (types, protocol)
  - State management (STATE.md, SUMMARY.md)
  - Configuration (config.json, modes)
- Link to authoritative source for each concept
- Add brief description for discoverability

**Must NOT do**:
- Duplicate content (only link)
- Add new concepts
- Change existing definitions

**Recommended Agent Profile**:
- **Category**: `writing`
  - Reason: Documentation organization task
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2 (with Tasks 4, 6)
- **Blocks**: Task 9
- **Blocked By**: None (can use current structure)

**References**:
- All files in `references/`, `templates/`, `workflows/`

**Acceptance Criteria**:
- [ ] File exists: `~/.config/opencode/get-shit-done/references/INDEX.md`
- [ ] All major concepts listed with links
- [ ] Categories clearly organized
- [ ] No duplicate explanations (links only)

**Commit**: YES
- Message: `docs(gsd): add references/INDEX.md concept index`
- Files: `references/INDEX.md`

---

### TODO 6: Improve gsd-help.md with Visual Categorization

**What to do**:
- Restructure gsd-help.md to show simplified view by default
- Add "GETTING STARTED" section with 3-4 essential commands
- Add "CORE WORKFLOW" section with main commands
- Add "/gsd-help full" option for complete reference
- Improve visual hierarchy with clear sections

**Must NOT do**:
- Remove any command documentation
- Change command descriptions
- Add new commands

**Recommended Agent Profile**:
- **Category**: `writing`
  - Reason: Documentation reorganization
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 2 (with Tasks 4, 5)
- **Blocks**: Task 9
- **Blocked By**: Task 3 (needs to include gsd-start)

**References**:
- `~/.config/opencode/command/gsd-help.md` - Current help file

**Acceptance Criteria**:
- [ ] Help shows simplified view with ~10 commands by default
- [ ] "GETTING STARTED" section includes /gsd-start
- [ ] Full reference available via "/gsd-help full"
- [ ] Visual hierarchy improved with clear sections

**Commit**: YES
- Message: `docs(gsd): improve gsd-help with visual categorization`
- Files: `command/gsd-help.md`

---

### TODO 7: Consolidate Duplicate Content

**What to do**:
- Identify content duplicated across files:
  - Checkpoint explanations in execute-plan and checkpoints
  - Git commit patterns in multiple places
  - State management in multiple workflows
- Choose single source of truth for each concept
- Replace duplicates with @references
- Add "See [concept] in [file]" redirects

**Must NOT do**:
- Remove unique context-specific details
- Break existing workflows
- Change authoritative sources arbitrarily

**Recommended Agent Profile**:
- **Category**: `unspecified-high`
  - Reason: Requires careful analysis of duplicates
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 3 (sequential)
- **Blocks**: Tasks 8, 9
- **Blocked By**: Task 4

**References**:
- Analysis from draft: `.sisyphus/drafts/gsd-analysis.md`

**Acceptance Criteria**:
- [ ] No concept explained fully in more than one file
- [ ] All duplicates replaced with @references
- [ ] Content reduction of at least 10%

**Commit**: YES
- Message: `refactor(gsd): consolidate duplicate content`
- Files: Multiple workflow and reference files

---

### TODO 8: Add Deprecation Notices

**What to do**:
- Add deprecation notices to old monolithic files pointing to modules
- Add notices for any commands that will be simplified in future
- Document migration path for existing users
- Update VERSION to reflect changes

**Must NOT do**:
- Actually remove deprecated items
- Break backward compatibility
- Remove functionality

**Recommended Agent Profile**:
- **Category**: `quick`
  - Reason: Simple notice additions
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 3 (after Task 7)
- **Blocks**: Task 9
- **Blocked By**: Task 7

**References**:
- All refactored files

**Acceptance Criteria**:
- [ ] Deprecation notices in original execute-plan.md header
- [ ] Deprecation notices in original checkpoints.md header
- [ ] VERSION file updated

**Commit**: YES
- Message: `chore(gsd): add deprecation notices for modular migration`
- Files: Modified files with notices, VERSION

---

### TODO 9: Validation and Documentation

**What to do**:
- Test all commands still work:
  - /gsd-new-project (dry run)
  - /gsd-start (various contexts)
  - /gsd-help (both modes)
  - /gsd-progress
- Verify no broken @references
- Update CHANGELOG.md with changes
- Create migration guide for users

**Must NOT do**:
- Skip any validation step
- Assume things work without testing
- Leave documentation outdated

**Recommended Agent Profile**:
- **Category**: `unspecified-high`
  - Reason: Comprehensive validation required
- **Skills**: `[]`

**Parallelization**:
- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 4 (final)
- **Blocks**: None (final task)
- **Blocked By**: Tasks 5, 6, 8

**References**:
- All modified files
- `~/.config/opencode/get-shit-done/CHANGELOG.md`

**Acceptance Criteria**:
- [ ] All commands execute without errors
- [ ] No broken @references found
- [ ] CHANGELOG.md updated with all changes
- [ ] Migration guide created

**Commit**: YES
- Message: `docs(gsd): complete v1.12 improvements validation`
- Files: `CHANGELOG.md`, migration docs

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `refactor(gsd): split execute-plan.md` | workflows/execute-plan*.md | wc -l < 500 each |
| 2 | `refactor(gsd): split checkpoints.md` | references/checkpoints*.md | wc -l < 400 each |
| 3 | `feat(gsd): add /gsd-start` | command/gsd-start.md | file exists |
| 4 | `refactor(gsd): update @references` | multiple | grep validates |
| 5 | `docs(gsd): add INDEX.md` | references/INDEX.md | file exists |
| 6 | `docs(gsd): improve gsd-help` | command/gsd-help.md | sections exist |
| 7 | `refactor(gsd): consolidate content` | multiple | reduced lines |
| 8 | `chore(gsd): add deprecation notices` | multiple, VERSION | notices present |
| 9 | `docs(gsd): complete validation` | CHANGELOG.md | all tests pass |

---

## Success Criteria

### Verification Commands
```bash
# No file exceeds 600 lines
find ~/.config/opencode/get-shit-done -name "*.md" -exec wc -l {} + | awk '$1 > 600 {print}'
# Should return empty

# All new files exist
ls ~/.config/opencode/get-shit-done/workflows/execute-plan-*.md
ls ~/.config/opencode/get-shit-done/references/checkpoints-*.md
ls ~/.config/opencode/get-shit-done/command/gsd-start.md
ls ~/.config/opencode/get-shit-done/references/INDEX.md

# No broken references (basic check)
grep -r "@.*\.md" ~/.config/opencode/get-shit-done --include="*.md" | grep -v "^#" | head -20
```

### Final Checklist
- [ ] All files under 600 lines
- [ ] /gsd-start command works in all contexts
- [ ] /gsd-help shows simplified view
- [ ] INDEX.md documents all concepts
- [ ] No duplicate content remains
- [ ] CHANGELOG.md updated
- [ ] VERSION incremented to 1.12.0
