#!/usr/bin/env bash
# GitOps drift gate (old `make check WHAT=gitops` selector).
set -euo pipefail
cd "$(dirname "$0")/../.."
exec uv run python scripts/check/gitops.py run
