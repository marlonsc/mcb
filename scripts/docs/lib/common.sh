#!/bin/bash
# =============================================================================
# Shared Library for Documentation Scripts
# =============================================================================
# Source this file to get logging, path resolution, counters, and utilities.
# Usage: source "$SCRIPT_DIR/lib/common.sh"
# =============================================================================

# Prevent double-sourcing
[[ -n "${__DOCS_LIB_LOADED:-}" ]] && return
__DOCS_LIB_LOADED=1

# =============================================================================
# Path Resolution
# =============================================================================

# Get the directory of the script that sourced this library
_get_caller_dir() {
    local source="${BASH_SOURCE[1]}"
    [[ -n "$source" ]] && cd "$(dirname "$source")" && pwd
}

# Initialize standard paths - call after sourcing
init_paths() {
    # Get this library's absolute location
    local lib_dir
    local lib_source="${BASH_SOURCE[0]}"

    # Handle relative paths
    if [[ "$lib_source" != /* ]]; then
        lib_source="$PWD/$lib_source"
    fi
    lib_dir="$(cd "$(dirname "$lib_source")" && pwd)"

    # If SCRIPT_DIR not set, derive from lib location (lib is in scripts/docs/lib)
    if [[ -z "${SCRIPT_DIR:-}" ]]; then
        SCRIPT_DIR="$(dirname "$lib_dir")"
    fi

    # Derive PROJECT_ROOT from SCRIPT_DIR (scripts/docs -> project root)
    if [[ -z "${PROJECT_ROOT:-}" ]]; then
        PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
    fi

    DOCS_DIR="${DOCS_DIR:-$PROJECT_ROOT/docs}"
    ADR_DIR="${ADR_DIR:-$DOCS_DIR/adr}"
    BOOK_DIR="${BOOK_DIR:-$PROJECT_ROOT/book}"

    export SCRIPT_DIR PROJECT_ROOT DOCS_DIR ADR_DIR BOOK_DIR
}

# =============================================================================
# Colors
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# =============================================================================
# Logging Functions
# =============================================================================

log_info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $*"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $*"; }
log_header()  { echo -e "${PURPLE}[HEADER]${NC} $*"; }
log_debug()   { [[ -n "${DEBUG:-}" ]] && echo -e "${CYAN}[DEBUG]${NC} $*"; }

# =============================================================================
# Counter Management
# =============================================================================

# Global counters (use functions to avoid subshell issues)
declare -g __ERRORS=0
declare -g __WARNINGS=0

reset_counters() {
    __ERRORS=0
    __WARNINGS=0
}

inc_errors() {
    ((__ERRORS++)) || true
}

inc_warnings() {
    ((__WARNINGS++)) || true
}

get_errors() {
    echo "$__ERRORS"
}

get_warnings() {
    echo "$__WARNINGS"
}

# =============================================================================
# Summary Reporting
# =============================================================================

print_summary() {
    local title="${1:-Validation Summary}"
    echo
    log_info "$title:"
    echo "  Errors: $__ERRORS"
    echo "  Warnings: $__WARNINGS"

    if [[ $__ERRORS -gt 0 ]]; then
        log_error "FAILED"
        return 1
    elif [[ $__WARNINGS -gt 0 ]]; then
        log_warning "PASSED with warnings"
        return 0
    else
        log_success "PASSED"
        return 0
    fi
}

exit_with_summary() {
    local title="${1:-Validation Summary}"
    print_summary "$title"
    local result=$?
    exit $result
}

# =============================================================================
# File/Directory Checks
# =============================================================================

check_file() {
    local file="$1"
    local description="${2:-}"

    if [[ ! -f "$file" ]]; then
        log_error "Missing file: $file${description:+ ($description)}"
        inc_errors
        return 1
    fi
    log_success "File exists: $(basename "$file")"
    return 0
}

check_directory() {
    local dir="$1"
    local description="${2:-}"

    if [[ ! -d "$dir" ]]; then
        log_error "Missing directory: $dir${description:+ ($description)}"
        inc_errors
        return 1
    fi
    log_success "Directory exists: $(basename "$dir")"
    return 0
}

check_executable() {
    local cmd="$1"
    local install_hint="${2:-}"

    if ! command -v "$cmd" &>/dev/null; then
        log_warning "$cmd not found${install_hint:+. Install: $install_hint}"
        return 1
    fi
    return 0
}

require_executable() {
    local cmd="$1"
    local install_hint="${2:-}"

    if ! command -v "$cmd" &>/dev/null; then
        log_error "$cmd is required but not found${install_hint:+. Install: $install_hint}"
        inc_errors
        return 1
    fi
    return 0
}

# =============================================================================
# ADR Utilities
# =============================================================================

# Valid ADR statuses
ADR_VALID_STATUSES=("Proposed" "Accepted" "Rejected" "Deprecated" "Superseded by ADR-")

validate_adr_status() {
    local status="$1"
    for valid in "${ADR_VALID_STATUSES[@]}"; do
        [[ "$status" == "$valid"* ]] && return 0
    done
    return 1
}

get_adr_status() {
    local adr_file="$1"
    grep -A1 "^## Status" "$adr_file" 2>/dev/null | tail -1 | tr -d '\r' | sed 's/^[[:space:]]*//'
}

get_adr_number() {
    local filename="$1"
    basename "$filename" .md | cut -d- -f1
}

is_adr_file() {
    local filename="$1"
    [[ "$(basename "$filename")" =~ ^[0-9]{3}-.*\.md$ ]]
}

# =============================================================================
# Markdown Utilities
# =============================================================================

find_markdown_files() {
    local dir="${1:-$DOCS_DIR}"
    find "$dir" -name "*.md" -type f \
        -not -path "*/vendor/*" \
        -not -path "*/third-party/*" \
        -not -path "*/node_modules/*" \
        -not -path "*/target/*" \
        2>/dev/null
}

has_trailing_whitespace() {
    local file="$1"
    grep -q '[[:space:]]$' "$file" 2>/dev/null
}

count_markdown_issues() {
    local file="$1"
    local issues=0

    # Trailing whitespace
    has_trailing_whitespace "$file" && ((issues++))

    # Multiple consecutive blank lines
    grep -q -E '^$' "$file" && ((issues++))

    echo "$issues"
}

# =============================================================================
# Dry-run Support
# =============================================================================

DRY_RUN="${DRY_RUN:-false}"

is_dry_run() {
    [[ "$DRY_RUN" == "true" ]] || [[ "$DRY_RUN" == "1" ]]
}

run_or_echo() {
    if is_dry_run; then
        echo "[DRY-RUN] Would execute: $*"
    else
        "$@"
    fi
}

# =============================================================================
# Metrics (source from extract-metrics.sh)
# =============================================================================

source_metrics() {
    local metrics_script="$SCRIPT_DIR/extract-metrics.sh"
    if [[ -x "$metrics_script" ]]; then
        # shellcheck source=/dev/null
        source <("$metrics_script" --env 2>/dev/null) || true
    fi
}

# =============================================================================
# Auto-initialize when sourced
# =============================================================================

init_paths
