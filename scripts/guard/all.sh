#!/usr/bin/env bash
# Validate the repository-owned Rust rule catalog with the native scanner.
set -euo pipefail
cd "$(dirname "$0")/../.."
exec ast-grep scan --config sgconfig.yml
