#!/bin/bash

# MCP Context Browser - Documentation Synchronization
# Detects code changes and suggests documentation updates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

log_header() {
    echo -e "${PURPLE}[SYNC]${NC} $1"
}

# Counters
needs_update=0
auto_updated=0

# Check if files have changed since last documentation generation
check_file_changes() {
    local source_file="$1"
    local doc_file="$2"
    local doc_marker="$3"

    if [ ! -f "$doc_file" ]; then
        log_warning "Documentation file missing: $doc_file"
        ((needs_update++))
        return
    fi

    # Check if source file is newer than doc file
    if [ "$source_file" -nt "$doc_file" ]; then
        log_warning "Source file newer than docs: $source_file"
        ((needs_update++))

        # Check if it contains the auto-generation marker
        if grep -q "$doc_marker" "$doc_file" 2>/dev/null; then
            log_info "Auto-generated file can be updated: $doc_file"
            ((auto_updated++))
        fi
    fi
}

# Check for new public APIs
check_new_apis() {
    log_header "🔍 Checking for new public APIs"

    # Find new public functions/structs/traits not documented
    local new_apis=$(find "$PROJECT_ROOT/src" -name "*.rs" -type f -exec grep -l "^pub " {} \; | head -5)

    for file in $new_apis; do
        local filename=$(basename "$file" .rs)
        local module=$(echo "$file" | sed "s|$PROJECT_ROOT/src/||" | sed 's|/.*||')

        # Check if this module has documentation
        if [ ! -f "$PROJECT_ROOT/docs/modules/$module.md" ]; then
            log_warning "New module detected: $module"
            ((needs_update++))
        fi
    done
}

# Check for version consistency
check_version_consistency() {
    log_header "📋 Checking version consistency"

    local cargo_version=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')
    local docs_versions=$(grep -r "version.*0\." "$PROJECT_ROOT/docs/" | wc -l)

    log_info "Cargo.toml version: $cargo_version"
    log_info "Documentation version references: $docs_versions"

    # Check if main docs files have correct version
    local main_files=("docs/README.md" "docs/architecture/ARCHITECTURE.md" "docs/user-guide/README.md")

    for file in "${main_files[@]}"; do
        if ! grep -q "$cargo_version" "$file" 2>/dev/null; then
            log_warning "Version mismatch in $file"
            ((needs_update++))
        fi
    done
}

# Check for provider documentation
check_provider_docs() {
    log_header "🤖 Checking provider documentation"

    # Get implemented providers
    local embedding_providers=$(ls "$PROJECT_ROOT/src/providers/embedding/" 2>/dev/null | grep '\.rs$' | grep -v mod | sed 's/\.rs$//' | sort)
    local vector_providers=$(ls "$PROJECT_ROOT/src/providers/vector_store/" 2>/dev/null | grep '\.rs$' | grep -v mod | sed 's/\.rs$//' | sort)

    # Check if all are documented in ARCHITECTURE.md
    for provider in $embedding_providers; do
        if ! grep -qi "$provider" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" 2>/dev/null; then
            log_warning "Undocumented embedding provider: $provider"
            ((needs_update++))
        fi
    done

    for provider in $vector_providers; do
        if ! grep -qi "$provider" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" 2>/dev/null; then
            log_warning "Undocumented vector store provider: $provider"
            ((needs_update++))
        fi
    done
}

# Check for routing system documentation
check_routing_docs() {
    log_header "🔀 Checking routing system documentation"

    if [ -d "$PROJECT_ROOT/src/providers/routing" ]; then
        local routing_modules=$(ls "$PROJECT_ROOT/src/providers/routing/" 2>/dev/null | wc -l)

        if [ "$routing_modules" -gt 5 ] && ! grep -qi "routing\|Routing\|circuit" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" 2>/dev/null; then
            log_warning "Routing system not documented in architecture"
            ((needs_update++))
        fi
    fi
}

# Generate sync report
generate_sync_report() {
    log_header "📊 Documentation Sync Report"

    echo "=========================================="
    echo "Documentation Synchronization Results"
    echo "=========================================="
    echo "Files needing manual review: $needs_update"
    echo "Auto-generated files to update: $auto_updated"
    echo "=========================================="

    if [ $needs_update -gt 0 ]; then
        log_warning "❌ Documentation sync issues found"
        echo
        echo "🔧 Recommended Actions:"
        echo "1. Run 'make docs-auto' to update auto-generated docs"
        echo "2. Review manually maintained docs for outdated information"
        echo "3. Check provider documentation in ARCHITECTURE.md"
        echo "4. Update version references if needed"
        echo "5. Add documentation for new modules/features"
        return 1
    elif [ $auto_updated -gt 0 ]; then
        log_info "⚠️  Auto-generated docs can be refreshed"
        echo
        echo "💡 Suggested Actions:"
        echo "1. Run 'make docs-auto' to refresh auto-generated content"
        return 0
    else
        log_success "✅ Documentation is synchronized"
        return 0
    fi
}

# Update auto-generated docs if needed
update_auto_docs() {
    if [ $auto_updated -gt 0 ]; then
        log_info "Updating auto-generated documentation..."
        make docs-auto >/dev/null 2>&1
        log_success "Auto-generated docs updated"
    fi
}

# Main execution
main() {
    log_header "🔄 MCP Context Browser - Documentation Synchronization"
    echo "======================================================="

    check_version_consistency
    check_provider_docs
    check_routing_docs
    check_new_apis

    # Check auto-generated files
    check_file_changes "$PROJECT_ROOT/src/lib.rs" "$PROJECT_ROOT/docs/api-reference.md" "Auto-generated API reference"
    check_file_changes "$PROJECT_ROOT/src/core/mod.rs" "$PROJECT_ROOT/docs/modules/core.md" "Auto-generated from source code"
    check_file_changes "$PROJECT_ROOT/Cargo.toml" "$PROJECT_ROOT/docs/implementation-status.md" "Auto-generated implementation status"

    generate_sync_report

    # Optionally update auto docs
    if [ "$1" = "--update" ]; then
        update_auto_docs
    fi
}

# Run main function
main "$@"