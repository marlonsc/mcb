#!/usr/bin/env python3
"""Scan docs/ for outdated content patterns."""

import os
import re
import sys
import argparse

import utils


def _process_lines(lines, rel_filepath, OUTDATED_PATTERNS, is_suppressed):
    issues_in_file = []
    for i, line in enumerate(lines, 1):
        # Skip whitespace, comments, code blocks start/end
        if (
            not line.strip()
            or line.strip().startswith("<!--")
            or line.strip().startswith("```")
        ):
            continue

        # Check line content
        for pattern, desc in OUTDATED_PATTERNS:
            # Use ignore case if pattern is lowercase
            flags = re.IGNORECASE if pattern.islower() else 0
            if re.search(pattern, line, flags) and not is_suppressed(line):
                issues_in_file.append((rel_filepath, i, desc, line.strip()[:80]))
    return issues_in_file


def _check_files(docs_dir, project_root):
    issues = []
    checked = 0

    # Patterns to flag
    OUTDATED_PATTERNS = [
        (r"v0\.1\.[0-9]+", "old version reference (v0.1.x)"),
        (r"shaku", "shaku DI (superseded by dill)"),
        (r"Shaku", "Shaku DI (superseded by dill)"),
        (r"inventory", "inventory crate (migrated to linkme)"),
        (r"rockets?", "Rocket web framework (migrated to Poem)"),
        (r"mcp-context-browser", "old project name (now mcb)"),
        (r"MCP Context Browser", "old project name (now Memory Context Browser / MCB)"),
        (r"mcb-adapters", "old crate name (removed/renamed)"),
        (r"mcb-core", "old crate name (split into mcb-domain + mcb-application)"),
        (r"CODEQL_SETUP", "reference to archived doc"),
    ]

    # Validation helpers
    def is_suppressed(line):
        return re.search(
            r"superseded|historical|migrated|referenc|deprecat|NOTE|dill|poem|linkme|previous|archived|legacy|renamed|removed",
            line,
            re.IGNORECASE,
        )

    md_files = utils.find_md_files(
        docs_dir, exclude_dirs={".git", "fixtures", "archive"}
    )

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked += 1

        try:
            with open(filepath, "r", encoding="utf-8") as fh:
                lines = fh.readlines()
        except Exception as e:
            print(f"Error reading {rel_filepath}: {e}")
            continue

        file_issues = _process_lines(
            lines, rel_filepath, OUTDATED_PATTERNS, is_suppressed
        )
        issues.extend(file_issues)

    return issues, checked


def main():
    """Main entry point for outdated documentation check."""
    parser = argparse.ArgumentParser(description="Check outdated content in docs.")
    parser.add_argument("--root", default=".", help="Project root directory")
    args = parser.parse_args()

    # Use project root from args if provided, otherwise detect
    project_root = os.path.abspath(args.root)
    if args.root == ".":
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, "docs")

    if not os.path.exists(docs_dir):
        print(f"Error: docs directory not found at {docs_dir}")
        sys.exit(1)

    issues, checked = _check_files(docs_dir, project_root)

    print(f"Checked {checked} files for outdated content.")

    if issues:
        print(f"Found {len(issues)} potential outdated references:")
        for fp, lineno, desc, content in sorted(issues):
            print(f"  {fp}:{lineno} [{desc}] {content}")
        # Return 0 for now as these are often false positives or acceptable history
        sys.exit(0)
    else:
        print("No outdated content found.")
        sys.exit(0)


if __name__ == "__main__":
    main()
