#!/usr/bin/env bash
# Full Rust surface sweep = lint gate + workspace suite, by composition.
# The commands live in their single-source scripts (lint.sh / test.sh).
set -euo pipefail
cd "$(dirname "$0")/.."
bash rust/lint.sh
bash rust/test.sh
