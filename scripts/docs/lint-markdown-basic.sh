#!/bin/bash

# MCP Context Browser - Basic Markdown Linting Script
# Basic markdown linting using native tools (no external dependencies)

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

# Check trailing whitespace
check_trailing_whitespace() {
    log_info "Checking trailing whitespace..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        if grep -q '[[:space:]]$' "$file"; then
            log_error "Trailing whitespace found in: $(basename "$file")"
            grep -n '[[:space:]]$' "$file" | head -5 | while read -r line; do
                log_error "  Line $line"
            done
            ((errors++))
        fi
    done

    if [ $errors -eq 0 ]; then
        log_success "No trailing whitespace found"
    fi
}

# Check multiple consecutive blank lines
check_multiple_blank_lines() {
    log_info "Checking multiple consecutive blank lines..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        if grep -q '\n\n\n' "$file"; then
            log_error "Multiple consecutive blank lines found in: $(basename "$file")"
            grep -n '\n\n\n' "$file" | head -3 | while read -r line; do
                log_error "  Line $line"
            done
            ((errors++))
        fi
    done

    if [ $errors -eq 0 ]; then
        log_success "No multiple consecutive blank lines found"
    fi
}

# Check unordered list consistency
check_unordered_lists() {
    log_info "Checking unordered list consistency..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Check for mixed list markers
        if grep -q '^[[:space:]]*\*[[:space:]]' "$file" && grep -q '^[[:space:]]*-[[:space:]]' "$file"; then
            log_warning "Mixed list markers (* and -) found in: $(basename "$file")"
            ((warnings++))
        fi
    done

    if [ $warnings -eq 0 ]; then
        log_success "Unordered list consistency OK"
    fi
}

# Check code blocks without language tags
check_code_blocks() {
    log_info "Checking code blocks..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Find fenced code blocks without language tags
        if grep -q '^```$' "$file"; then
            log_warning "Code blocks without language tags found in: $(basename "$file")"
            ((warnings++))
        fi
    done

    if [ $warnings -eq 0 ]; then
        log_success "Code block language tags OK"
    fi
}

# Check header levels
check_headers() {
    log_info "Checking header levels..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Check for skipped header levels (e.g., # followed by ###)
        local has_h1=$(grep -q '^# ' "$file" && echo "yes" || echo "no")
        local has_h3=$(grep -q '^### ' "$file" && echo "yes" || echo "no")

        if [ "$has_h3" = "yes" ] && [ "$has_h1" = "no" ]; then
            log_warning "Skipped header level (no # found before ###) in: $(basename "$file")"
            ((warnings++))
        fi
    done

    if [ $warnings -eq 0 ]; then
        log_success "Header levels OK"
    fi
}

# Main execution
main() {
    log_info "MCP Context Browser - Basic Markdown Linting"
    log_info "==========================================="

    check_trailing_whitespace
    check_multiple_blank_lines
    check_unordered_lists
    check_code_blocks
    check_headers

    echo
    log_info "Linting Summary:"
    echo "  Errors: $errors"
    echo "  Warnings: $warnings"

    if [ $errors -gt 0 ]; then
        log_error "Markdown linting failed with $errors errors"
        exit 1
    elif [ $warnings -gt 0 ]; then
        log_warning "Markdown linting passed with $warnings warnings"
        exit 0
    else
        log_success "Markdown linting passed with no issues"
        exit 0
    fi
}

# Run main function
main "$@"