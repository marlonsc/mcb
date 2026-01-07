#!/bin/bash

# MCP Context Browser - Documentation Structure Validation
# Validates that documentation follows the required structure and standards

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

# Check if directory exists
check_directory() {
    local dir="$1"
    local description="$2"

    if [ ! -d "$dir" ]; then
        log_error "Missing directory: $dir ($description)"
        ((errors++))
        return 1
    else
        log_success "Directory exists: $dir"
        return 0
    fi
}

# Check if file exists
check_file() {
    local file="$1"
    local description="$2"

    if [ ! -f "$file" ]; then
        log_error "Missing file: $file ($description)"
        ((errors++))
        return 1
    else
        log_success "File exists: $file"
        return 0
    fi
}

# Validate ADR format
validate_adr() {
    local adr_file="$1"
    local filename=$(basename "$adr_file")

    # Skip if not an ADR file
    if [[ ! "$filename" =~ ^[0-9]+\.md$ ]]; then
        return 0
    fi

    log_info "Validating ADR: $filename"

    # Check for required sections
    local required_sections=("## Status" "## Context" "## Decision")
    local has_errors=false

    for section in "${required_sections[@]}"; do
        if ! grep -q "^$section" "$adr_file"; then
            log_error "ADR $filename missing required section: $section"
            has_errors=true
        fi
    done

    # Check status values
    local status_line=$(grep "^## Status" "$adr_file" | head -1)
    if [[ -n "$status_line" ]]; then
        local status_value=$(echo "$status_line" | sed 's/## Status//' | tr -d '\n\r' | sed 's/^[[:space:]]*//')
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

    if [ "$has_errors" = true ]; then
        ((errors++))
    else
        log_success "ADR $filename format is valid"
    fi
}

# Validate documentation structure
validate_structure() {
    log_info "Validating documentation structure..."

    # Check main directories
    check_directory "$PROJECT_ROOT/docs" "Main documentation directory"
    check_directory "$PROJECT_ROOT/docs/user-guide" "User guide documentation"
    check_directory "$PROJECT_ROOT/docs/developer" "Developer documentation"
    check_directory "$PROJECT_ROOT/docs/architecture" "Architecture documentation"
    check_directory "$PROJECT_ROOT/docs/operations" "Operations documentation"
    check_directory "$PROJECT_ROOT/docs/templates" "Documentation templates"

    # Check key files
    check_file "$PROJECT_ROOT/docs/README.md" "Documentation index"
    check_file "$PROJECT_ROOT/docs/user-guide/README.md" "User guide"
    check_file "$PROJECT_ROOT/docs/developer/CONTRIBUTING.md" "Contributing guide"
    check_file "$PROJECT_ROOT/docs/developer/ROADMAP.md" "Development roadmap"
    check_file "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" "Architecture overview"
    check_file "$PROJECT_ROOT/docs/operations/DEPLOYMENT.md" "Deployment guide"
    check_file "$PROJECT_ROOT/docs/operations/CHANGELOG.md" "Changelog"
    check_file "$PROJECT_ROOT/docs/templates/adr-template.md" "ADR template"

    # Check architecture subdirectories
    check_directory "$PROJECT_ROOT/docs/adr" "Architecture Decision Records"
    check_directory "$PROJECT_ROOT/docs/diagrams" "Architecture diagrams"

    # Validate ADR files
    if [ -d "$PROJECT_ROOT/docs/adr" ]; then
        for adr_file in "$PROJECT_ROOT/docs/adr"/*.md; do
            if [ -f "$adr_file" ]; then
                validate_adr "$adr_file"
            fi
        done
    fi

    # Check for orphaned files in root
    local orphaned_files=("ARCHITECTURE.md" "CONTRIBUTING.md" "ROADMAP.md" "DEPLOYMENT.md" "CHANGELOG.md")
    for file in "${orphaned_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$file" ]; then
            log_error "Orphaned documentation file in root: $file (should be in docs/)"
            ((errors++))
        fi
    done
}

# Validate file permissions
validate_permissions() {
    log_info "Validating file permissions..."

    # Check script permissions
    local scripts=("generate-diagrams.sh" "validate-structure.sh" "validate-links.sh" "check-sync.sh" "validate-adrs.sh" "create-adr.sh" "generate-index.sh")

    for script in "${scripts[@]}"; do
        local script_path="$SCRIPT_DIR/$script"
        if [ -f "$script_path" ]; then
            if [ ! -x "$script_path" ]; then
                log_error "Script not executable: $script"
                ((errors++))
            else
                log_success "Script executable: $script"
            fi
        else
            log_warning "Script missing: $script"
            ((warnings++))
        fi
    done
}

# Main execution
main() {
    log_info "MCP Context Browser - Documentation Structure Validation"
    log_info "======================================================="

    validate_structure
    validate_permissions

    echo
    log_info "Validation Summary:"
    echo "  Errors: $errors"
    echo "  Warnings: $warnings"

    if [ $errors -gt 0 ]; then
        log_error "Documentation structure validation FAILED"
        exit 1
    else
        log_success "Documentation structure validation PASSED"
        if [ $warnings -gt 0 ]; then
            log_warning "Some warnings were found - review recommended"
        fi
        exit 0
    fi
}

# Run main function
main "$@"