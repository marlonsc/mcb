#!/usr/bin/env bash
# Full mcb-validate structural validation (the `all` selector and the
# bare-verb discovery entrypoint; WHAT=quick runs the reduced pass).
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh validate full
