#!/bin/bash

# MCP Context Browser - ADR Validation
# Validates Architecture Decision Records format and consistency

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

# Validate ADR format
validate_adr_format() {
    local adr_file="$1"
    local filename=$(basename "$adr_file")

    log_info "Validating ADR: $filename"

    # Check filename format (should be NNN-title.md)
    if [[ ! "$filename" =~ ^[0-9]{3}-[a-z0-9-]+\.md$ ]]; then
        log_error "ADR filename format incorrect: $filename (should be NNN-title.md)"
        ((errors++))
        return
    fi

    # Extract ADR number
    local adr_num=$(echo "$filename" | cut -d'-' -f1)

    # Check ADR number in title
    local first_line=$(head -1 "$adr_file" | tr -d '\r')
    if ! echo "$first_line" | grep -q "^# ADR $adr_num:"; then
        log_error "ADR number mismatch in $filename"
        log_error "Expected ADR number: $adr_num"
        log_error "Found: $first_line"
        ((errors++))
    fi

    # Check required sections
    local required_sections=("## Status" "## Context" "## Decision")
    local has_errors=false

    for section in "${required_sections[@]}"; do
        if ! grep -q "^$section" "$adr_file"; then
            log_error "ADR $filename missing required section: $section"
            has_errors=true
        fi
    done

    # Check status values
    local status_line=$(grep -A1 "^## Status" "$adr_file" | tail -1 | tr -d '\r' | sed 's/^[[:space:]]*//')
    if [[ -n "$status_line" ]]; then
        local status_value="$status_line"
        local valid_statuses=("Proposed" "Accepted" "Rejected" "Deprecated" "Superseded by ADR-")

        local is_valid=false
        for valid_status in "${valid_statuses[@]}"; do
            if [[ "$status_value" == "$valid_status"* ]]; then
                is_valid=true
                break
            fi
        done

        if [ "$is_valid" = false ]; then
            log_error "ADR $filename has invalid status: '$status_value'"
            has_errors=true
        fi
    fi

    # Check for consequences section if status is Accepted
    if [[ "$status_value" == "Accepted"* ]]; then
        if ! grep -q "^## Consequences" "$adr_file"; then
            log_warning "ADR $filename (Accepted) missing Consequences section"
            ((warnings++))
        fi
    fi

    # Check for alternatives section if status is Accepted
    if [[ "$status_value" == "Accepted"* ]]; then
        if ! grep -q "^## Alternatives Considered" "$adr_file"; then
            log_warning "ADR $filename (Accepted) missing Alternatives Considered section"
            ((warnings++))
        fi
    fi

    if [ "$has_errors" = false ]; then
        log_success "ADR $filename format is valid"
    else
        ((errors++))
    fi
}

# Check ADR numbering consistency
check_adr_numbering() {
    log_info "Checking ADR numbering consistency..."

    local adr_files=$(ls "$PROJECT_ROOT/docs/architecture/adr" | grep -E '^[0-9]+\.md$' | sort -V)
    local expected_num=1

    for adr_file in $adr_files; do
        local actual_num=$(echo "$adr_file" | sed 's/\.md$//')
        if [ "$actual_num" -ne "$expected_num" ]; then
            log_error "ADR numbering gap: expected $expected_num, found $actual_num"
            ((errors++))
        fi
        ((expected_num++))
    done

    log_success "ADR numbering is consistent"
}

# Check ADR references in documentation
check_adr_references() {
    log_info "Checking ADR references in documentation..."

    local adr_files=$(ls "$PROJECT_ROOT/docs/architecture/adr" | grep -E '^[0-9]+\.md$')
    local arch_doc="$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"

    if [ -f "$arch_doc" ]; then
        for adr_file in $adr_files; do
            local adr_num=$(echo "$adr_file" | sed 's/\.md$//')
            if ! grep -q "ADR $adr_num" "$arch_doc"; then
                log_warning "ADR $adr_num not referenced in architecture documentation"
                ((warnings++))
            fi
        done
    fi

    log_success "ADR reference check completed"
}

# Main execution
main() {
    log_info "MCP Context Browser - ADR Validation"
    log_info "===================================="

    local adr_dir="$PROJECT_ROOT/docs/architecture/adr"

    if [ ! -d "$adr_dir" ]; then
        log_error "ADR directory not found: $adr_dir"
        exit 1
    fi

    # Validate each ADR file
    for adr_file in "$adr_dir"/*.md; do
        if [ -f "$adr_file" ] && [[ "$(basename "$adr_file")" != "README.md" ]] && [[ "$(basename "$adr_file")" != "TEMPLATE.md" ]]; then
            validate_adr_format "$adr_file"
        fi
    done

    # Check numbering consistency
    check_adr_numbering

    # Check references
    check_adr_references

    echo
    log_info "ADR Validation Summary:"
    echo "  Errors: $errors"
    echo "  Warnings: $warnings"

    if [ $errors -gt 0 ]; then
        log_error "ADR validation FAILED"
        exit 1
    else
        log_success "ADR validation PASSED"
        if [ $warnings -gt 0 ]; then
            log_warning "Some warnings found - review recommended"
        fi
        exit 0
    fi
}

# Run main function
main "$@"