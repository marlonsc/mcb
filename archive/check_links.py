#!/usr/bin/env python3
"""Scan docs/ for broken internal markdown links."""

import os
import re

docs_dir = "docs"
broken = []
checked = 0

for root, dirs, files in os.walk(docs_dir):
    if "fixtures" in root:
        continue
    for f in files:
        if not f.endswith(".md"):
            continue
        filepath = os.path.join(root, f)
        checked += 1
        with open(filepath) as fh:
            content = fh.read()
        # Strip HTML comments to avoid flagging placeholder links in templates
        content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
        links = re.findall(r"\[([^\]]*)\]\(([^)#]+?)(?:#[^)]+)?\)", content)
        for text, link in links:
            if link.startswith("http") or link.startswith("mailto:"):
                continue
            if link.startswith("/"):
                target = link[1:]
            else:
                target = os.path.normpath(os.path.join(os.path.dirname(filepath), link))
            if not os.path.exists(target):
                broken.append((filepath, text, link, target))

print(f"Checked {checked} docs")
print(f"Found {len(broken)} broken links:")
for fp, text, link, target in sorted(broken):
    print(f"  {fp}: [{text}]({link}) -> {target}")
