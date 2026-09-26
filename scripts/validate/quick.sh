#!/usr/bin/env bash
# Quick mcb-validate structural validation (old `make check WHAT=validate QUICK=1`).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh validate quick
