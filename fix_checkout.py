#!/usr/bin/env python3
"""Add persist-credentials: false to checkout steps that don't already have it."""

import re


def add_persist_credentials(filepath, skip_indices=None):
    """Add persist-credentials: false to checkout steps.

    skip_indices: set of 0-based checkout occurrence indices to skip
    """
    if skip_indices is None:
        skip_indices = set()

    with open(filepath) as f:
        content = f.read()

    lines = content.split("\n")
    new_lines = []
    checkout_idx = 0
    i = 0

    while i < len(lines):
        line = lines[i]

        if "actions/checkout@" in line and "- uses:" in line:
            indent_match = re.match(r"^(\s*)-", line)
            indent = indent_match.group(1) if indent_match else "      "

            if checkout_idx in skip_indices:
                new_lines.append(line)
                i += 1
            else:
                new_lines.append(line)
                # Check if next line has 'with:'
                if i + 1 < len(lines) and lines[i + 1].strip().startswith("with:"):
                    new_lines.append(lines[i + 1])
                    i += 2
                    # Insert persist-credentials right after 'with:'
                    new_lines.append(f"{indent}  persist-credentials: false")
                else:
                    # Add 'with:' block
                    new_lines.append(f"{indent}  with:")
                    new_lines.append(f"{indent}    persist-credentials: false")
                    i += 1

            checkout_idx += 1
        else:
            new_lines.append(line)
            i += 1

    with open(filepath, "w") as f:
        f.write("\n".join(new_lines))

    print(
        f"Fixed {filepath} ({checkout_idx} checkout steps found, {len(skip_indices)} skipped)"
    )


# ci.yml: all checkout steps get persist-credentials: false
add_persist_credentials("/home/marlonsc/mcb-check-fix/.github/workflows/ci.yml")

# codeql.yml: all checkout steps
add_persist_credentials("/home/marlonsc/mcb-check-fix/.github/workflows/codeql.yml")

# docs.yml: skip update-docs job (index 1 - needs to push)
add_persist_credentials(
    "/home/marlonsc/mcb-check-fix/.github/workflows/docs.yml", skip_indices={1}
)

# release.yml: all checkout steps
add_persist_credentials("/home/marlonsc/mcb-check-fix/.github/workflows/release.yml")

# retag-on-merge.yml: skip (needs to push tags)
add_persist_credentials(
    "/home/marlonsc/mcb-check-fix/.github/workflows/retag-on-merge.yml"
)
