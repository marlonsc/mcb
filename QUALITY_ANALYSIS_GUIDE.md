# Quality Analysis Tool Usage Guide

## Overview

`scripts/analyze_qlty.py` - Unified SARIF analyzer for `qlty check` and `qlty smells` reports with powerful filtering and reporting capabilities.

## Quick Start

### Basic Analysis

```bash
# Analyze checks only
python3 scripts/analyze_qlty.py --type checks --summary

# Analyze code smells only
python3 scripts/analyze_qlty.py --type smells --summary

# Analyze both together
python3 scripts/analyze_qlty.py --type both --summary
```

### Generate Reports

```bash
# Markdown report for checks
python3 scripts/analyze_qlty.py --type checks --markdown CHECKS_REPORT.md

# JSON export for automation
python3 scripts/analyze_qlty.py --type checks --json checks.json

# Combined report with both formats
python3 scripts/analyze_qlty.py --type both --markdown FULL_REPORT.md --json data.json
```

## Filtering Capabilities

### By Severity

```bash
# Only errors
python3 scripts/analyze_qlty.py --type checks --severity error --summary

# Only warnings
python3 scripts/analyze_qlty.py --type checks --severity warning --summary

# Only notes
python3 scripts/analyze_qlty.py --type checks --severity note --summary
```

### By Category

```bash
# GitHub Actions security issues only
python3 scripts/analyze_qlty.py --type checks --category zizmor --summary

# Formatting issues only
python3 scripts/analyze_qlty.py --type checks --category rustfmt --summary

# Dependency vulnerabilities only
python3 scripts/analyze_qlty.py --type checks --category osv-scanner --summary
```

### By Rule

```bash
# Unpinned GitHub Actions
python3 scripts/analyze_qlty.py --type checks --rule unpinned-uses --summary

# Cache poisoning issues
python3 scripts/analyze_qlty.py --type checks --rule cache-poisoning --summary

# Code duplication
python3 scripts/analyze_qlty.py --type smells --rule similar-code --summary
```

### By File

```bash
# All issues in CI workflow
python3 scripts/analyze_qlty.py --type checks --file ".github/workflows/ci.yml" --summary

# All issues in validate crate
python3 scripts/analyze_qlty.py --type both --file "crates/mcb-validate/*" --summary
```

## Current Project Status

### Checks Summary (134 total)

| Category | Count | % | Severity Breakdown |
|----------|-------|---|--------------------|
| **rustfmt** | 85 | 63.4% | 85 NOTE |
| **zizmor** | 44 | 32.8% | 24 ERROR, 20 WARNING |
| **osv-scanner** | 5 | 3.7% | 5 WARNING |

### Top Issues to Address

#### 🔴 Critical (24 errors)

1. **Unpinned GitHub Actions** (15 errors)
   - Files: `.github/workflows/*.yml`
   - Fix: Pin actions to SHA digests
   
2. **Cache Poisoning Risk** (6 errors)
   - Files: `.github/workflows/*.yml`
   - Fix: Add cache key validation

3. **Security Issues** (3 errors)
   - Dangerous triggers (1)
   - Bot conditions (1)
   - Excessive permissions (1)

#### 🟠 High Priority (25 warnings)

1. **Artipacked Warnings** (20 warnings)
   - Files: `.github/workflows/*.yml`
   - Fix: Review artifact upload/download patterns

2. **Dependency Vulnerabilities** (5 warnings)
   - File: `Cargo.lock`
   - Fix: Update vulnerable dependencies

#### 🔵 Low Priority (85 notes)

1. **Rustfmt Formatting** (85 notes)
   - Files: Various Rust source files
   - Fix: Run `cargo fmt`

## Recommended Workflow

### 1. Initial Assessment

```bash
# Get high-level overview
python3 scripts/analyze_qlty.py --type both --summary

# Generate detailed report
python3 scripts/analyze_qlty.py --type both --markdown FULL_ANALYSIS.md
```

### 2. Prioritize Critical Issues

```bash
# Focus on errors first
python3 scripts/analyze_qlty.py --type checks --severity error --markdown CRITICAL_FIXES.md

# Review what needs immediate attention
cat CRITICAL_FIXES.md
```

### 3. Category-Based Fixing

```bash
# Fix GitHub Actions security (zizmor errors)
python3 scripts/analyze_qlty.py --type checks --category zizmor --severity error --summary

# Fix dependency vulnerabilities
python3 scripts/analyze_qlty.py --type checks --category osv-scanner --summary

# Address code quality (smells)
python3 scripts/analyze_qlty.py --type smells --severity warning --summary
```

### 4. File-Based Fixing

```bash
# Fix all issues in CI workflow
python3 scripts/analyze_qlty.py --type checks --file ".github/workflows/ci.yml" --markdown ci_fixes.md

# Verify fixes worked
qlty check .github/workflows/ci.yml
python3 scripts/analyze_qlty.py --type checks --file ".github/workflows/ci.yml" --summary
```

## Integration with Beads

```bash
# Create issues for critical problems
python3 scripts/analyze_qlty.py --type checks --severity error --json errors.json

# Then manually or via script:
bd create --title="Fix unpinned GitHub Actions (15 issues)" --type=task --priority=0
bd create --title="Fix cache poisoning risks (6 issues)" --type=task --priority=0
bd create --title="Update vulnerable dependencies (5 issues)" --type=task --priority=1
```

## Output Formats

### Summary (Console)

- Quick overview with emoji indicators
- Severity breakdown with percentages
- Top 10 rules and files
- Ideal for quick checks

### Markdown Report

- Complete analysis with tables
- Organized by severity
- Grouped by rule with examples
- Ready for documentation or PRs

### JSON Export

- Machine-readable format
- Ideal for automation
- Can be processed by other tools
- Includes all metadata

## Examples from Current Project

### Find All Zizmor Errors

```bash
$ python3 scripts/analyze_qlty.py --type checks --category zizmor --severity error --summary

📊 ANALYSIS SUMMARY: 24 issues

## By Severity
🔴 ERROR      24 (100.0%)

## Top 10 Rules
    15 ( 62.5%)  zizmor:zizmor/unpinned-uses
     6 ( 25.0%)  zizmor:zizmor/cache-poisoning
     1 (  4.2%)  zizmor:zizmor/dangerous-triggers

## Top 10 Files
     9  .github/workflows/ci.yml
     6  .github/workflows/docs.yml
     5  .github/workflows/release.yml
```

### Combined Analysis

```bash
$ python3 scripts/analyze_qlty.py --type both --summary

📊 ANALYSIS SUMMARY: 512 issues

## By Severity
🔴 ERROR      24 (  4.7%)
🟠 WARNING   403 ( 78.7%)
🔵 NOTE       85 ( 16.6%)

## Top 10 Rules
   139 ( 27.1%)  qlty:similar-code
    91 ( 17.8%)  qlty:function-complexity
    91 ( 17.8%)  qlty:nested-control-flow
```

## Advanced Usage

### Custom Input Files

```bash
python3 scripts/analyze_qlty.py \
  --type both \
  --checks-file custom_checks.sarif \
  --smells-file custom_smells.sarif \
  --summary
```

### Pipeline Integration

```bash
# Run in CI/CD pipeline
qlty check --all > qlty.check.lst 2>&1
python3 scripts/analyze_qlty.py --type checks --json results.json

# Fail build if errors exist
if jq -e '.by_severity.ERROR > 0' results.json; then
  echo "❌ Quality check failed: errors found"
  exit 1
fi
```

## Tips

1. **Start with errors**: `--severity error` focuses on must-fix issues
2. **Category filtering**: Tackle one tool at a time (rustfmt → zizmor → osv-scanner)
3. **File-based iteration**: Fix one file completely before moving to next
4. **Combine with beads**: Create issues for systematic tracking
5. **Export JSON**: Use for automation and tracking over time

## Next Steps

Based on current analysis:

1. Fix 24 zizmor errors (GitHub Actions security) - HIGH PRIORITY
2. Update 5 vulnerable dependencies - MEDIUM PRIORITY  
3. Run `cargo fmt` to fix 85 rustfmt notes - LOW PRIORITY
4. Address code smells (139 duplications) - TECHNICAL DEBT

---

**Generated:** 2026-02-08  
**Tool Version:** analyze_qlty.py v1.0
