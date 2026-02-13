#!/usr/bin/env python3
"""Scan docs/ for references to source paths that don't exist."""

import os
import re
import sys
import argparse


def main():
    """Main entry point for checking broken source references in documentation.

    # Code Smells
    TODO(qlty): Function with high complexity (count = 37).
    """
    parser = argparse.ArgumentParser(
        description="Check broken source references in docs."
    )
    parser.add_argument("--root", default=".", help="Project root directory")
    args = parser.parse_args()

    project_root = os.path.abspath(args.root)
    docs_dir = os.path.join(project_root, "docs")

    if not os.path.exists(docs_dir):
        print(f"Error: docs directory not found at {docs_dir}")
        sys.exit(1)

    issues = []
    checked = 0

    for root, dirs, files in os.walk(docs_dir):
        if "fixtures" in root or ".git" in root:
            continue
        for f in files:
            if not f.endswith(".md"):
                continue

            filepath = os.path.join(root, f)
            rel_filepath = os.path.relpath(filepath, project_root)
            checked += 1

            try:
                with open(filepath, "r", encoding="utf-8") as file:
                    content = file.read()
            except Exception as e:
                print(f"Error reading {rel_filepath}: {e}")
                continue

            # Strip comments
            content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)

            # Find references like `crates/mcb-xxx/src/...`
            refs = re.findall(r"`(crates/[^`]+)`", content)

            for ref in refs:
                # Basic filter: exclude things that look like commands or snippets with spaces
                if " " in ref or "(" in ref or "::" in ref or "..." in ref:
                    continue

                # Resolve checking existence
                target = os.path.join(project_root, ref.rstrip("/"))

                # Check directly or check if it's a file without extension (directories)
                # Also try checking if it's a Rust file reference without .rs extension (common in docs)
                if not os.path.exists(target):
                    # TODO(qlty): Deeply nested control flow (level = 5).
                    if not os.path.exists(target + ".rs"):
                        # TODO(qlty): Deeply nested control flow (level = 5).
                        issues.append((rel_filepath, ref))

    print(f"Checked source refs in {checked} docs")

    if issues:
        print(f"Found {len(issues)} broken source references:")
        for fp, ref in sorted(set(issues)):
            print(f"  {fp}: `{ref}` -> Not found")
        sys.exit(1)
    else:
        print("No broken source references found.")
        sys.exit(0)


if __name__ == "__main__":
    main()
