#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

MAX_LINES="${MAX_LINES:-200}"
STALE_DAYS="${STALE_DAYS:-30}"

python3 - "$PROJECT_ROOT" "$MAX_LINES" "$STALE_DAYS" <<'PY'
import os
import re
import sys
import time

root = sys.argv[1]
max_lines = int(sys.argv[2])
stale_days = int(sys.argv[3])

targets = ["context", "docs/context"]
ref_pattern = re.compile(r"`((?:context|docs/context)/[^`]+\.md)`")
md_link_pattern = re.compile(r"\[[^\]]*\]\(((?:context|docs/context)/[^)#]+)(?:#[^)]+)?\)")

errors = []
warnings = []

def scan_file(path: str) -> None:
    rel = os.path.relpath(path, root)
    with open(path, encoding="utf-8") as f:
        lines = f.readlines()
    text = "".join(lines)

    if len(lines) > max_lines:
        errors.append(f"{rel}: {len(lines)} lines (max {max_lines})")

    age_days = (time.time() - os.path.getmtime(path)) / 86400.0
    if age_days > stale_days:
        warnings.append(f"{rel}: stale ({age_days:.1f} days old)")

    refs = set(ref_pattern.findall(text))
    refs.update(md_link_pattern.findall(text))
    for ref in sorted(refs):
        full = os.path.join(root, ref)
        if not os.path.exists(full):
            errors.append(f"{rel}: broken reference -> {ref}")


checked = 0
for target in targets:
    base = os.path.join(root, target)
    if not os.path.isdir(base):
        warnings.append(f"{target}: directory missing")
        continue
    for dirpath, _, filenames in os.walk(base):
        for filename in filenames:
            if filename.endswith(".md"):
                checked += 1
                scan_file(os.path.join(dirpath, filename))

print(f"Checked {checked} markdown files across context targets")

if warnings:
    print("\nWarnings:")
    for warning in warnings:
        print(f"- {warning}")

if errors:
    print("\nErrors:")
    for error in errors:
        print(f"- {error}")
    sys.exit(1)

print("\nOK: context validation passed")
PY
