#!/usr/bin/env bash
# Documentation Markdown checks and repairs use the provisioned formatter.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

main() {
    local action="${1:-lint}"
    local dry_run="${2:-}"
    local config_file="$PROJECT_ROOT/.markdownlint.json"
    local ignore_file="$PROJECT_ROOT/.markdownlintignore"
    local args=("$DOCS_DIR/")

    if [[ -f "$config_file" ]]; then
        args+=(--config "$config_file")
    fi
    if [[ -f "$ignore_file" ]]; then
        args+=(--ignore-path "$ignore_file")
    fi

    require_executable markdownlint
    case "$action" in
        lint|check)
            [[ -z "$dry_run" ]] || {
                log_error "Unexpected argument: $dry_run"
                return 2
            }
            markdownlint "${args[@]}"
            ;;
        fix|autofix)
            if [[ -z "$dry_run" ]]; then
                markdownlint --fix "${args[@]}"
            elif [[ "$dry_run" == "--dry-run" ]]; then
                markdownlint "${args[@]}"
            else
                log_error "Unexpected argument: $dry_run"
                return 2
            fi
            ;;
        help|--help|-h)
            printf '%s\n' 'Usage: markdown.sh {lint|check|fix|autofix} [--dry-run]'
            ;;
        *)
            log_error "Unknown action: $action"
            return 2
            ;;
    esac
}

main "$@"
