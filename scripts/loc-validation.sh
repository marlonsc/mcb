#!/bin/bash
# scripts/loc-validation.sh — canonical measurement script
set -euo pipefail

TOKEI_CMD="tokei crates/ -t Rust -e '**/tests/**' -e '**/fixtures/**' -e '**/benches/**'"

echo "=== Source Code LOC (per-crate) ==="
for crate in mcb-domain mcb-application mcb-infrastructure mcb-providers mcb-server mcb-validate; do
	echo -n "$crate: "
	tokei "crates/$crate/src" -t Rust -e '**/tests/**' -o json | jq '.Rust.code'
done

echo "=== Aggregate Source LOC ==="
eval "$TOKEI_CMD -o json" | jq '.Rust.code'

echo "=== Test Count ==="
cargo test --workspace 2>&1 | grep "test result" | tail -1

echo "=== Compile Time ==="
cargo build --workspace --timings 2>&1 | grep "Finished"

echo "=== Clippy ==="
cargo clippy --workspace -- -D warnings 2>&1 | tail -3

echo "=== Doc Warnings ==="
cargo doc --workspace --no-deps 2>&1 | grep -c "warning" || echo "0"

echo "=== Quality Metrics ==="
echo -n "Production unwrap/expect: "
grep -rn '\.unwrap()\|\.expect(' crates/mcb-application/src/ crates/mcb-server/src/ crates/mcb-infrastructure/src/ --include="*.rs" | grep -v test | wc -l
echo -n "fn get_ in ports: "
grep -rn 'fn get_' crates/mcb-domain/src/ports/ --include="*.rs" | wc -l
echo -n "pub status: String: "
grep -rn 'pub status: String\|pub kind: String' crates/ --include="*.rs" | wc -l
echo -n "reqwest in domain: "
grep -rn 'reqwest' crates/mcb-domain/ --include="*.rs" | wc -l
