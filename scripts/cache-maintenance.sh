#!/usr/bin/env bash
# =============================================================================
# scripts/cache-maintenance.sh — Safe local compilation-cache maintenance.
#
# Usage:
#   scripts/cache-maintenance.sh          # DRY-RUN (reports only)
#   scripts/cache-maintenance.sh --apply  # Actually prune caches
#
# Keeps the MCB workspace build footprint predictable:
#   - Bounds sccache via SCCACHE_CACHE_SIZE (set in .cargo/config.toml).
#   - Removes stale artifacts from target/ with cargo-sweep.
#   - Reports before/after disk usage and sccache statistics.
#
# Safe to run at any time: the default mode only prints what would be done.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
APPLY=N

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
for arg in "$@"; do
    case "${arg}" in
        --apply|-a)
            APPLY=Y
            ;;
        --help|-h)
            cat <<'EOF'
Usage: scripts/cache-maintenance.sh [OPTIONS]

Safe local compilation-cache maintenance for the MCB workspace.

Options:
  --apply, -a   Actually prune caches (default is DRY-RUN)
  --help, -h    Show this help

Examples:
  # Report only — safe to run anytime
  scripts/cache-maintenance.sh

  # Prune caches (destructive — removes stale build artifacts)
  scripts/cache-maintenance.sh --apply
EOF
            exit 0
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log_info()  { printf "[INFO]  %s\n" "$1"; }
log_warn()  { printf "[WARN]  %s\n" "$1" >&2; }
log_ok()    { printf "[OK]    %s\n" "$1"; }

human_bytes() {
    local bytes=$1
    if command -v numfmt &>/dev/null; then
        numfmt --to=iec-i --suffix=B "${bytes}" 2>/dev/null || echo "${bytes}B"
    else
        echo "${bytes}B"
    fi
}

disk_usage() {
    local path=$1
    if [ -d "${path}" ]; then
        du -sb "${path}" 2>/dev/null | cut -f1 || echo 0
    else
        echo 0
    fi
}

# ---------------------------------------------------------------------------
# sccache maintenance
# ---------------------------------------------------------------------------
maintenance_sccache() {
    printf "\n=== sccache maintenance ======================================================\n"

    if ! command -v sccache &>/dev/null; then
        log_warn "sccache not found in PATH; skipping sccache maintenance."
        return 0
    fi

    log_info "Current sccache statistics:"
    sccache --show-stats || true

    log_info "Restarting sccache server so SCCACHE_CACHE_SIZE takes effect..."
    if [ "${APPLY}" = "Y" ]; then
        sccache --stop-server 2>/dev/null || true
        sccache --start-server 2>/dev/null || true
        log_ok "sccache server restarted."
    else
        log_info "DRY-RUN: would restart sccache server."
    fi
}

# ---------------------------------------------------------------------------
# target/ maintenance via cargo-sweep
# ---------------------------------------------------------------------------
maintenance_target() {
    printf "\n=== target/ maintenance ======================================================\n"

    local target_dir="${PROJECT_ROOT}/target"
    local before after
    before=$(disk_usage "${target_dir}")
    log_info "target/ size before: $(human_bytes "${before}")"

    if ! command -v cargo-sweep &>/dev/null; then
        log_warn "cargo-sweep not found in PATH."
        log_info "Install with: cargo install cargo-sweep --locked"
        return 0
    fi

    if [ "${APPLY}" = "Y" ]; then
        log_info "Pruning target/ artifacts older than 30 days..."
        cargo-sweep sweep -t 30 -r "${target_dir}" || log_warn "cargo-sweep sweep -t 30 returned non-zero"

        log_info "Pruning target/ down to a maximum of 50 GiB if needed..."
        cargo-sweep sweep -m 50GB -r "${target_dir}" || log_warn "cargo-sweep sweep -m 50GB returned non-zero"

        after=$(disk_usage "${target_dir}")
        log_ok "target/ size after: $(human_bytes "${after}") (freed $(human_bytes $((before - after))))"
    else
        log_info "DRY-RUN: would run:"
        log_info "  cargo-sweep sweep -t 30 -r ${target_dir}"
        log_info "  cargo-sweep sweep -m 50GB -r ${target_dir}"
    fi
}

# ---------------------------------------------------------------------------
# Report environment recommendations
# ---------------------------------------------------------------------------
print_recommendations() {
    printf "\n=== Recommended cache settings ==============================================\n"
    printf "These are now enforced automatically via .cargo/config.toml [env]:\n\n"
    printf "  SCCACHE_CACHE_SIZE=10G\n"
    printf "  CARGO_BUILD_JOBS=8\n"
    printf "  RAYON_NUM_THREADS=4\n"
    printf "\nRun this script weekly or when disk usage is high:\n"
    printf "  make check WHAT=optimize ACT=cache APPLY=Y\n"
    printf "=============================================================================\n"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    printf "\n"
    printf "╔══════════════════════════════════════════════════════════════════════════════╗\n"
    printf "║  MCB Cache Maintenance                                                       ║\n"
    printf "║  Mode: %-69s ║\n" "$([ "${APPLY}" = "Y" ] && printf "APPLY (prune)" || printf "DRY-RUN (report)")"
    printf "╚══════════════════════════════════════════════════════════════════════════════╝\n"

    cd "${PROJECT_ROOT}"
    maintenance_sccache
    maintenance_target
    print_recommendations

    printf "\n[INFO] Done.\n"
    if [ "${APPLY}" != "Y" ]; then
        printf "[INFO] No caches were pruned. Run with --apply to execute cleanup.\n"
    fi
}

main "$@"
