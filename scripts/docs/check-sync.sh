#!/bin/bash

# MCP Context Browser - Code-Documentation Synchronization Check
# Ensures documentation stays synchronized with code changes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Counters
errors=0
warnings=0

# Get version from Cargo.toml
get_version() {
    grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/'
}

# Check version consistency
check_version_consistency() {
    log_info "Checking version consistency..."

    local cargo_version=$(get_version)

    # Check README.md
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        if ! grep -q "$cargo_version" "$PROJECT_ROOT/README.md"; then
            log_error "Version $cargo_version not found in README.md"
            ((errors++))
        fi
    fi

    # Check docs README
    if [ -f "$PROJECT_ROOT/docs/README.md" ]; then
        if ! grep -q "$cargo_version" "$PROJECT_ROOT/docs/README.md"; then
            log_warning "Version $cargo_version not mentioned in docs/README.md"
            ((warnings++))
        fi
    fi

    log_success "Version consistency check completed"
}

# Check API consistency
check_api_consistency() {
    log_info "Checking API consistency..."

    # Check that main modules are documented
    local main_modules=("core" "providers" "services" "server" "config" "factory" "registry")

    for module in "${main_modules[@]}"; do
        if [ -d "$PROJECT_ROOT/src/$module" ]; then
            if [ ! -f "$PROJECT_ROOT/src/$module/mod.rs" ]; then
                log_error "Module $module missing mod.rs file"
                ((errors++))
            fi
        fi
    done

    log_success "API consistency check completed"
}

# Check diagram consistency with code
check_diagram_consistency() {
    log_info "Checking diagram consistency..."

    # Check that architecture diagrams reference actual modules
    local arch_diagram="$PROJECT_ROOT/docs/architecture/diagrams/system-context.puml"

    if [ -f "$arch_diagram" ]; then
        # Check for key components mentioned in diagrams
        local key_components=("MCP Context Browser" "Vector Database" "AI Provider" "Metadata Store")

        for component in "${key_components[@]}"; do
            if ! grep -q "$component" "$arch_diagram"; then
                log_warning "Component '$component' not found in system context diagram"
                ((warnings++))
            fi
        done
    fi

    log_success "Diagram consistency check completed"
}

# Check documentation coverage
check_doc_coverage() {
    log_info "Checking documentation coverage..."

    # Check that major features are documented
    local features=("semantic" "vector embeddings" "MCP protocol" "provider pattern")
    local arch_doc="$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"

    if [ -f "$arch_doc" ]; then
        for feature in "${features[@]}"; do
            if ! grep -i -q "$feature" "$arch_doc"; then
                log_warning "Feature '$feature' may not be documented in architecture"
                ((warnings++))
            fi
        done
    fi

    log_success "Documentation coverage check completed"
}

# Check for outdated references
check_outdated_references() {
    log_info "Checking for outdated references..."

    # Check for references to old file locations (files that were moved from root to docs/)
    local old_paths=("ARCHITECTURE.md" "CONTRIBUTING.md" "ROADMAP.md")

    for old_path in "${old_paths[@]}"; do
        # Check if any docs reference files in root that should be in docs/
        # Only flag references that don't include "docs/" or "../architecture/" path
        local bad_references=$(grep -r "$old_path" "$PROJECT_ROOT/docs/" 2>/dev/null | grep -v "docs/" | grep -v "\.\./architecture/" || true)

        if [ -n "$bad_references" ]; then
            log_warning "Found incorrect references to moved file: $old_path (should include docs/ path)"
            ((warnings++))
        fi
    done

    log_success "Outdated references check completed"
}

# Check ADR consistency
check_adr_consistency() {
    log_info "Checking ADR consistency..."

    local adr_dir="$PROJECT_ROOT/docs/architecture/adr"

    if [ -d "$adr_dir" ]; then
        # Check ADR numbering
        local expected_num=1
        local found_adrs=$(ls "$adr_dir" | grep -E '^[0-9]+\.md$' | sort -V)

        for adr_file in $found_adrs; do
            local actual_num=$(echo "$adr_file" | sed 's/\.md$//')
            if [ "$actual_num" -ne "$expected_num" ]; then
                log_error "ADR numbering gap or out of order: expected $expected_num, found $actual_num"
                ((errors++))
            fi
            ((expected_num++))
        done

        # Check that ADRs are referenced in architecture doc
        local arch_doc="$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"
        if [ -f "$arch_doc" ]; then
            for adr_file in $found_adrs; do
                local adr_num=$(echo "$adr_file" | sed 's/\.md$//')
                if ! grep -q "ADR $adr_num" "$arch_doc"; then
                    log_warning "ADR $adr_num not referenced in architecture documentation"
                    ((warnings++))
                fi
            done
        fi
    fi

    log_success "ADR consistency check completed"
}

# Main execution
main() {
    log_info "MCP Context Browser - Code-Documentation Synchronization Check"
    log_info "=============================================================="

    check_version_consistency
    check_api_consistency
    check_diagram_consistency
    check_doc_coverage
    check_outdated_references
    check_adr_consistency

    echo
    log_info "Synchronization Check Summary:"
    echo "  Errors: $errors"
    echo "  Warnings: $warnings"

    if [ $errors -gt 0 ]; then
        log_error "Code-documentation synchronization check FAILED"
        exit 1
    else
        log_success "Code-documentation synchronization check PASSED"
        if [ $warnings -gt 0 ]; then
            log_warning "Some warnings found - review recommended"
        fi
        exit 0
    fi
}

# Run main function
main "$@"