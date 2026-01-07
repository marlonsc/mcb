#!/bin/bash

# MCP Context Browser - Code vs Documentation Audit
# Comprehensive audit of source code against documentation

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
    echo -e "${PURPLE}[AUDIT]${NC} $1"
}

# Counters
errors=0
warnings=0
info_count=0

# Audit modules
audit_modules() {
    log_header "🔍 Auditing Modules Implementation vs Documentation"

    echo "📦 Implemented modules: chunking, config, core, daemon, factory, metrics, providers, registry, server, services, snapshot, sync"

    # Check specific modules that are likely undocumented
    local undocumented=""

    for mod in chunking daemon metrics snapshot sync; do
        if ! grep -qi "$mod" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" 2>/dev/null; then
            undocumented="$undocumented $mod"
        fi
    done

    if [ -z "$undocumented" ]; then
        log_success "✅ All major modules are documented"
    else
        log_warning "⚠️  Modules undocumented:$undocumented"
        ((warnings++))
    fi
    echo
}

# Audit providers
audit_providers() {
    log_header "🤖 Auditing Provider Implementations"

    # Get implemented providers
    local embedding_providers=$(ls "$PROJECT_ROOT/src/providers/embedding/" | grep '\.rs$' | sed 's/\.rs$//' | grep -v mod | sort)
    local vector_providers=$(ls "$PROJECT_ROOT/src/providers/vector_store/" | grep '\.rs$' | sed 's/\.rs$//' | grep -v mod | sort)

    echo "🧠 Embedding providers:"
    for provider in $embedding_providers; do
        echo "   • $provider"
    done
    echo

    echo "💾 Vector store providers:"
    for provider in $vector_providers; do
        echo "   • $provider"
    done
    echo

    # Check documentation coverage
    local total_providers=$(echo "$embedding_providers $vector_providers" | wc -w)
    local documented_providers=0

    for provider in $embedding_providers; do
        if grep -qi "$provider" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
            ((documented_providers++))
        fi
    done

    for provider in $vector_providers; do
        if grep -qi "$provider" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
            ((documented_providers++))
        fi
    done

    if [ $documented_providers -eq $total_providers ]; then
        log_success "✅ All $total_providers providers are documented"
    else
        log_warning "⚠️  $((total_providers - documented_providers)) providers undocumented"
        ((warnings++))
    fi
    echo
}

# Audit features from Cargo.toml
audit_features() {
    log_header "⚙️ Auditing Cargo.toml Features vs Documentation"

    # Get key dependencies that indicate features
    local key_features=(
        "tokio.*full" "Async runtime"
        "axum" "HTTP server/metrics API"
        "prometheus" "Metrics collection"
        "sysinfo" "System monitoring"
        "fs2" "File locking/sync"
        "hostname" "Lock metadata"
        "tree-sitter" "Code parsing/AST"
        "governor" "Rate limiting"
        "health" "Health checks"
    )

    local undocumented_features=()

    for ((i=0; i<${#key_features[@]}; i+=2)); do
        local feature="${key_features[i]}"
        local description="${key_features[i+1]}"

        if grep -q "$feature" "$PROJECT_ROOT/Cargo.toml"; then
            echo "✅ $description ($feature)"
            if ! grep -qi "$description" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
                undocumented_features+=("$description")
            fi
        fi
    done
    echo

    if [ ${#undocumented_features[@]} -eq 0 ]; then
        log_success "✅ All implemented features are documented"
    else
        log_error "❌ ${#undocumented_features[@]} features implemented but undocumented:"
        for feature in "${undocumented_features[@]}"; do
            echo "   • $feature"
        done
        ((errors++))
    fi
    echo
}

# Audit routing system
audit_routing() {
    log_header "🔀 Auditing Routing System Implementation"

    local routing_modules=$(ls "$PROJECT_ROOT/src/providers/routing/" | grep -E '\.rs$' | sed 's/\.rs$//' | sort)
    echo "🎯 Routing modules:"
    for mod in $routing_modules; do
        echo "   • $mod"
    done
    echo

    # Check if routing is documented
    if grep -qi "routing\|Routing" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
        log_success "✅ Routing system is documented"
    else
        log_error "❌ Routing system is implemented but not documented"
        ((errors++))
    fi
    echo
}

# Audit hybrid search
audit_hybrid_search() {
    log_header "🔍 Auditing Hybrid Search Implementation"

    if [ -f "$PROJECT_ROOT/src/core/hybrid_search.rs" ]; then
        echo "✅ Hybrid search module exists"

        if grep -qi "hybrid\|Hybrid\|BM25" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
            log_success "✅ Hybrid search is documented"
        else
            log_error "❌ Hybrid search is implemented but not documented"
            ((errors++))
        fi
    else
        log_info "ℹ️  Hybrid search not implemented yet"
    fi
    echo
}

# Audit daemon
audit_daemon() {
    log_header "👻 Auditing Daemon Implementation"

    if [ -f "$PROJECT_ROOT/src/daemon/mod.rs" ]; then
        echo "✅ Daemon module exists"

        if grep -qi "daemon\|Daemon" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
            log_success "✅ Daemon is documented"
        else
            log_error "❌ Daemon is implemented but not documented"
            ((errors++))
        fi
    else
        log_info "ℹ️  Daemon not implemented yet"
    fi
    echo
}

# Audit sync system
audit_sync() {
    log_header "🔒 Auditing Sync System Implementation"

    if [ -d "$PROJECT_ROOT/src/sync" ]; then
        echo "✅ Sync system exists"

        if grep -qi "sync\|Sync\|lockfile" "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"; then
            log_success "✅ Sync system is documented"
        else
            log_error "❌ Sync system is implemented but not documented"
            ((errors++))
        fi
    else
        log_info "ℹ️  Sync system not implemented yet"
    fi
    echo
}

# Generate audit report
generate_report() {
    log_header "📊 Audit Summary Report"

    echo "=========================================="
    echo "Code vs Documentation Audit Results"
    echo "=========================================="
    echo "Errors: $errors"
    echo "Warnings: $warnings"
    echo "Info items: $info_count"
    echo "=========================================="

    if [ $errors -gt 0 ]; then
        log_error "❌ Audit FAILED: $errors critical issues found"
        echo
        echo "🔧 Recommended Actions:"
        echo "1. Document missing features in ARCHITECTURE.md"
        echo "2. Update provider documentation"
        echo "3. Add routing system documentation"
        echo "4. Document hybrid search capabilities"
        echo "5. Add daemon and sync system docs"
        return 1
    elif [ $warnings -gt 0 ]; then
        log_warning "⚠️  Audit PASSED with warnings: $warnings items to review"
        return 0
    else
        log_success "✅ Audit PASSED: All code features are documented"
        return 0
    fi
}

# Main execution
main() {
    log_header "🚀 MCP Context Browser - Code vs Documentation Audit"
    echo "======================================================"

    audit_modules
    audit_providers
    audit_features
    audit_routing
    audit_hybrid_search
    audit_daemon
    audit_sync

    generate_report
}

# Run main function
main "$@"