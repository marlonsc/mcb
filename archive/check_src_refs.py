#!/usr/bin/env python3
"""Scan docs/ for references to source paths that don't exist."""

import os
import re

docs_dir = "docs"
issues = []
checked = 0

for root, dirs, files in os.walk(docs_dir):
    if "fixtures" in root or "archive" in root:
        continue
    for f in files:
        if not f.endswith(".md"):
            continue
        filepath = os.path.join(root, f)
        checked += 1
        with open(filepath) as fh:
            content = fh.read()
        # Find source code references like `crates/mcb-xxx/src/...`
        src_refs = re.findall(r"`(crates/[^`]+)`", content)
        for ref in src_refs:
            # Only check file/dir references, not code snippets
            if " " in ref or "(" in ref or "::" in ref:
                continue
            target = ref.rstrip("/")
            if not os.path.exists(target) and not os.path.exists(target + ".rs"):
                issues.append((filepath, ref, target))

print(f"Checked {checked} docs")
print(f"Found {len(issues)} broken source references:")
for fp, ref, target in sorted(set(issues)):
    print(f"  {fp}: {ref}")
