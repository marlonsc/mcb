#!/usr/bin/env bash
# Rust lint surface — SINGLE SOURCE of the fmt/clippy gate commands.
# Consumers (custom.mk pre-check hook, rust/all.sh composition) invoke this
# script; the commands are declared here and nowhere else.
set -euo pipefail
cd "$(dirname "$0")/../.."
bash scripts/lib/mcb.sh run cargo fmt --all -- --check
bash scripts/lib/mcb.sh run cargo clippy --all-targets -- -D warnings
