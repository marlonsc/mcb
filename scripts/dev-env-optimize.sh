#!/usr/bin/env bash
# =============================================================================
# dev-env-optimize.sh — Detect and clean up duplicate resource-heavy processes
# in multi-session development environments.
#
# Usage:
#   scripts/dev-env-optimize.sh          # DRY-RUN (reports only)
#   scripts/dev-env-optimize.sh --apply  # Actually kill duplicates
#
# Designed for MCB's multi-agent setup where each session spawns its own
# rust-analyzer + Serena MCP server, quickly exhausting 62GB RAM.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT=$(readlink -f "${PROJECT_ROOT:-${SCRIPT_DIR}/..}") || {
    printf "[ERROR] Unable to resolve project root.\n" >&2
    exit 1
}
PROC_ROOT="${PROC_ROOT:-/proc}"
KILL_COMMAND="${KILL_COMMAND:-kill}"
APPLY=N
KEEP_RA="${KEEP_RA:-1}"          # max rust-analyzer instances to keep per project
KEEP_SERENA="${KEEP_SERENA:-2}"  # max Serena MCP servers to keep per project
KEEP_CARGO="${KEEP_CARGO:-1}"    # max cargo processes to keep (zombie detection)
CARGO_ZOMBIE_MIN="${CARGO_ZOMBIE_MIN:-30}" # cargo processes older than this (minutes) are flagged

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
Usage: scripts/dev-env-optimize.sh [OPTIONS]

Detect and optionally kill duplicate rust-analyzer, Serena MCP server,
and stale cargo processes that accumulate across multiple agent sessions.

Options:
  --apply, -a   Actually kill processes (default is DRY-RUN)
  --help, -h    Show this help message

Environment:
  KEEP_RA       Max rust-analyzer instances to keep per project (default: 1)
  KEEP_SERENA   Max Serena MCP servers to keep per project (default: 2)
  KEEP_CARGO    Max cargo processes to keep (default: 1)

Examples:
  # Report only — safe to run anytime
  scripts/dev-env-optimize.sh

  # Clean up duplicates (destructive — kills processes)
  scripts/dev-env-optimize.sh --apply
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
log_kill()  { printf "[KILL]  %s\n" "$1"; }
log_skip()  { printf "[SKIP]  %s\n" "$1"; }

human_bytes() {
    local bytes=$1
    if command -v numfmt &>/dev/null; then
        numfmt --to=iec-i --suffix=B "${bytes}" 2>/dev/null || echo "${bytes}B"
    else
        echo "${bytes}B"
    fi
}

process_belongs_to_project() {
    local pid=$1 command_line=${2:-} process_cwd
    process_cwd=$(readlink -f "${PROC_ROOT}/${pid}/cwd" 2>/dev/null || true)
    case "${process_cwd}" in
        "${PROJECT_ROOT}"|"${PROJECT_ROOT}"/*) return 0 ;;
    esac
    case " ${command_line} " in
        *" ${PROJECT_ROOT} "*|*" ${PROJECT_ROOT}/"*) return 0 ;;
    esac
    return 1
}

project_pids() {
    local pattern=$1 pid command_line
    while read -r pid command_line; do
        [ -n "${pid}" ] || continue
        process_belongs_to_project "${pid}" "${command_line}" && printf '%s\n' "${pid}"
    done < <(pgrep -a -f "${pattern}" 2>/dev/null || true)
    return 0
}

terminate_process() { "${KILL_COMMAND}" -TERM "$1"; }

# ---------------------------------------------------------------------------
# Resource report
# ---------------------------------------------------------------------------
print_resource_report() {
    printf "\n=== Resource Report =========================================================\n"
    if command -v free &>/dev/null; then
        free -h
    fi
    printf "=============================================================================\n"
}

# ---------------------------------------------------------------------------
# Kill duplicate rust-analyzer instances
# ---------------------------------------------------------------------------
kill_duplicate_rust_analyzers() {
    printf "\n--- rust-analyzer duplicate cleanup -----------------------------------------\n"
    local pids
    pids=$(project_pids 'rust.analyzer|rust-analyzer')

    if [ -z "${pids}" ]; then
        log_info "No rust-analyzer processes found."
        return
    fi

    local total_count
    total_count=$(echo "${pids}" | wc -l)
    log_info "Found ${total_count} rust-analyzer process(es)."

    # Sort by start time (most recent first) via pid (higher = more recent in most cases)
    # For a more accurate sort, we use ps etime, but pid is a reasonable proxy
    local sorted_pids
    sorted_pids=$(echo "${pids}" | sort -rn)

    local keep_count=0
    local kill_count=0

    while IFS= read -r pid; do
        [ -z "${pid}" ] && continue
        if [ "${keep_count}" -lt "${KEEP_RA}" ]; then
            log_skip "Keeping PID ${pid} (rust-analyzer)"
            keep_count=$((keep_count + 1))
        else
            if [ "${APPLY}" = "Y" ]; then
                log_kill "PID ${pid} (rust-analyzer) — duplicate"
                terminate_process "${pid}" 2>/dev/null || log_warn "Failed to kill PID ${pid}"
            else
                log_skip "Would kill PID ${pid} (rust-analyzer) — DRY-RUN"
            fi
            kill_count=$((kill_count + 1))
        fi
    done <<< "${sorted_pids}"

    if [ "${kill_count}" -gt 0 ]; then
        if [ "${APPLY}" = "Y" ]; then
            log_info "Killed ${kill_count} duplicate rust-analyzer instance(s)."
        else
            log_info "Would kill ${kill_count} duplicate rust-analyzer instance(s). Use --apply to execute."
        fi
    fi
}

# ---------------------------------------------------------------------------
# Kill duplicate Serena MCP servers
# ---------------------------------------------------------------------------
kill_duplicate_serena_servers() {
    printf "\n--- Serena MCP server duplicate cleanup -------------------------------------\n"
    local pids
    pids=$(project_pids 'serena.*start-mcp-server')

    if [ -z "${pids}" ]; then
        log_info "No Serena MCP server processes found."
        return
    fi

    local total_count
    total_count=$(echo "${pids}" | wc -l)
    log_info "Found ${total_count} Serena MCP server process(es)."

    local sorted_pids
    sorted_pids=$(echo "${pids}" | sort -rn)

    local keep_count=0
    local kill_count=0

    while IFS= read -r pid; do
        [ -z "${pid}" ] && continue
        if [ "${keep_count}" -lt "${KEEP_SERENA}" ]; then
            log_skip "Keeping PID ${pid} (Serena MCP server)"
            keep_count=$((keep_count + 1))
        else
            if [ "${APPLY}" = "Y" ]; then
                log_kill "PID ${pid} (Serena MCP server) — duplicate"
                terminate_process "${pid}" 2>/dev/null || log_warn "Failed to kill PID ${pid}"
            else
                log_skip "Would kill PID ${pid} (Serena MCP server) — DRY-RUN"
            fi
            kill_count=$((kill_count + 1))
        fi
    done <<< "${sorted_pids}"

    if [ "${kill_count}" -gt 0 ]; then
        if [ "${APPLY}" = "Y" ]; then
            log_info "Killed ${kill_count} duplicate Serena MCP server instance(s)."
        else
            log_info "Would kill ${kill_count} duplicate Serena MCP server instance(s). Use --apply to execute."
        fi
    fi
}

# ---------------------------------------------------------------------------
# Flag stale cargo processes
# ---------------------------------------------------------------------------
flag_stale_cargo() {
    printf "\n--- Stale cargo process check -----------------------------------------------\n"
    local procs
    procs=$(ps -eo pid,etime,cmd 2>/dev/null | awk '$3 == "cargo" {print $1, $2, $3}' || true)

    if [ -z "${procs}" ]; then
        log_info "No cargo processes found."
        return
    fi

    while IFS= read -r line; do
        [ -z "${line}" ] && continue
        local pid elapsed cmd
        pid=$(echo "${line}" | awk '{print $1}')
        elapsed=$(echo "${line}" | awk '{print $2}')
        cmd=$(echo "${line}" | awk '{print $3}')

        process_belongs_to_project "${pid}" "${line}" || continue

        # Parse elapsed time (format: [[dd-]hh:]mm:ss)
        local minutes=0
        if [[ "${elapsed}" =~ - ]]; then
            # Has days
            local days
            days=$(echo "${elapsed}" | cut -d'-' -f1)
            minutes=$((days * 24 * 60))
            elapsed=$(echo "${elapsed}" | cut -d'-' -f2)
        fi

        local h m s
        if [[ "${elapsed}" =~ :.*: ]]; then
            # Has hours
            h=$(echo "${elapsed}" | cut -d':' -f1)
            m=$(echo "${elapsed}" | cut -d':' -f2)
            minutes=$((minutes + h * 60 + m))
        else
            m=$(echo "${elapsed}" | cut -d':' -f1)
            minutes=$((minutes + m))
        fi

        if [ "${minutes}" -ge "${CARGO_ZOMBIE_MIN}" ]; then
            log_warn "PID ${pid} (${cmd}) running for ${minutes} min — possibly stale"
            if [ "${APPLY}" = "Y" ]; then
                log_kill "PID ${pid} (${cmd}) — stale cargo process"
                terminate_process "${pid}" 2>/dev/null || log_warn "Failed to kill PID ${pid}"
            fi
        fi
    done <<< "${procs}"
}

# ---------------------------------------------------------------------------
# Print environment recommendations
# ---------------------------------------------------------------------------
print_env_recommendations() {
    printf "\n=== Recommended Environment Variables ======================================\n"
    printf "Export these in your shell or .bashrc for consistent optimization:\n\n"
    printf "  export CARGO_BUILD_JOBS=8\n"
    printf "  export RAYON_NUM_THREADS=4\n"
    printf "  export RA_LOG=error\n"
    printf "\nThese are now also set in .cargo/config.toml [env] for automatic use.\n"
    printf "=============================================================================\n"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    printf "\n"
    printf "╔══════════════════════════════════════════════════════════════════════════════╗\n"
    printf "║  MCB Dev Environment Optimizer                                               ║\n"
    printf "║  Mode: %s                                                                  ║\n" "$([ "${APPLY}" = "Y" ] && printf "APPLY (destructive)" || printf "DRY-RUN (safe)")"
    printf "╚══════════════════════════════════════════════════════════════════════════════╝\n"

    print_resource_report
    kill_duplicate_rust_analyzers
    kill_duplicate_serena_servers
    flag_stale_cargo
    print_env_recommendations

    printf "\n[INFO] Done.\n"
    if [ "${APPLY}" != "Y" ]; then
        printf "[INFO] No processes were killed. Run with --apply to execute cleanup.\n"
    fi
}

main "$@"
