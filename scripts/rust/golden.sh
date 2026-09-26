#!/usr/bin/env bash
# Golden e2e test (old `make test WHAT=golden` selector; MATCH passes through).
set -euo pipefail
cd "$(dirname "$0")/../.."
match="${MATCH:-golden}"
bash scripts/lib/mcb.sh run cargo test -p mcb-server --test e2e "$match"
