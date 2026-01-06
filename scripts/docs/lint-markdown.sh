#!/bin/bash

# MCP Context Browser - Markdown Linting Script
# Robust markdown linting with fallbacks

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

# Check if markdownlint-cli is available
check_markdownlint() {
    if command -v markdownlint >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Fallback markdown linting using basic tools
lint_markdown_fallback() {
    log_info "Using fallback markdown linting..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        local filename=$(basename "$file")

        # Check for trailing whitespace
        if grep -q '[[:space:]]$' "$file"; then
            log_error "Trailing whitespace found in $filename"
            ((errors++))
        fi

        # Check for multiple consecutive blank lines (more than 2)
        if grep -q '\n\n\n' "$file"; then
            log_warning "Multiple consecutive blank lines in $filename (should be max 2)"
            ((warnings++))
        fi

        # Check for inconsistent list markers (mix of * and -)
        if grep -q '^[[:space:]]*\*[[:space:]]' "$file" && grep -q '^[[:space:]]*-[[:space:]]' "$file"; then
            log_warning "Mixed list markers (* and -) in $filename"
            ((warnings++))
        fi

        # Check for headers without proper spacing
        if grep -A1 '^#.*' "$file" | grep -q '^[^#[:space:]]'; then
            log_warning "Header without proper spacing in $filename"
            ((warnings++))
        fi

        # Check for code blocks without language tags
        if grep -q '^```$' "$file"; then
            log_warning "Code block without language tag in $filename"
            ((warnings++))
        fi
    done
}

# Main markdownlint execution
lint_markdown_full() {
    log_info "Using markdownlint-cli for comprehensive linting..."

    if markdownlint "$PROJECT_ROOT/docs/" --config "$PROJECT_ROOT/.markdownlint.json"; then
        log_success "Markdown linting passed"
        return 0
    else
        log_error "Markdown linting failed"
        return 1
    fi
}

# Main execution
main() {
    log_info "MCP Context Browser - Markdown Linting"
    log_info "======================================"

    local use_fallback=false

    if check_markdownlint; then
        log_info "markdownlint-cli found, using full linting"
        if ! lint_markdown_full; then
            exit 1
        fi
    else
        log_warning "markdownlint-cli not found, using fallback linting"
        lint_markdown_fallback
        use_fallback=true
    fi

    echo
    log_info "Linting Summary:"
    if [ "$use_fallback" = true ]; then
        echo "  Mode: Fallback (basic checks)"
        echo "  Errors: $errors"
        echo "  Warnings: $warnings"

        if [ $errors -gt 0 ]; then
            log_error "Found $errors errors. Run 'make fix-md' to auto-fix."
            exit 1
        elif [ $warnings -gt 0 ]; then
            log_warning "Found $warnings warnings. Consider running 'make fix-md'."
            exit 0
        else
            log_success "No issues found with fallback linting."
        fi
    else
        echo "  Mode: Full (markdownlint-cli)"
        log_success "All checks passed."
    fi
}

# Run main function
main "$@"