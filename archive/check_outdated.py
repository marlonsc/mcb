#!/usr/bin/env python3
"""Scan docs/ for outdated content patterns."""

import os
import re

docs_dir = "docs"
issues = []

OUTDATED_PATTERNS = [
    (r"v0\.1\.[0-9]+", "old version reference (v0.1.x)"),
    (r"shaku", "shaku DI (superseded by dill in ADR-029)"),
    (r"Shaku", "Shaku DI (superseded by dill in ADR-029)"),
    (r"inventory", "inventory crate (migrated to linkme in ADR-023)"),
    (r"rockets?(?:\s|,|\.)", "Rocket web framework (migrated to Poem in ADR-026)"),
    (r"0\.1\.3", "old v0.1.3 version reference"),
    (r"mcp-context-browser", "old project name (now mcb)"),
    (r"MCP Context Browser", "old project name (now Memory Context Browser / MCB)"),
    (r"mcb-adapters", "old crate name (removed/renamed)"),
    (r"mcb-core", "old crate name (split into mcb-domain + mcb-application)"),
    (r"mcb-cli", "old crate name (removed)"),
    (r"mcb-admin", "old module (merged into mcb-server)"),
    (r"CODEQL_SETUP", "reference to archived doc"),
]

for root, dirs, files in os.walk(docs_dir):
    if "fixtures" in root or "archive" in root:
        continue
    for f in files:
        if not f.endswith(".md"):
            continue
        filepath = os.path.join(root, f)
        with open(filepath) as fh:
            lines = fh.readlines()
        for i, line in enumerate(lines, 1):
            # Skip markdownlint directives, code blocks, and HTML comments
            if line.strip().startswith("<!--") or line.strip().startswith("```"):
                continue
            for pattern, desc in OUTDATED_PATTERNS:
                if re.search(
                    pattern, line, re.IGNORECASE if pattern[0].islower() else 0
                ):
                    issues.append((filepath, i, desc, line.strip()[:100]))

print(f"Found {len(issues)} potentially outdated references:")
for fp, lineno, desc, content in sorted(issues):
    print(f"  {fp}:{lineno} [{desc}] {content}")
