#!/usr/bin/env python3
"""Add persist-credentials: false to checkout steps that don't already have it."""

import re


def _extract_indent(line):
    """Extract indentation from a YAML line."""
    indent_match = re.match(r"^(\s*)-", line)
    return indent_match.group(1) if indent_match else "      "


def _should_add_credentials(checkout_idx, skip_indices):
    """Check if we should add persist-credentials to this checkout."""
    return checkout_idx not in skip_indices


def _process_checkout_with_block(lines, i, indent, new_lines):
    """Process a checkout step that already has a 'with:' block."""
    new_lines.append(lines[i + 1])
    new_lines.append(f"{indent}  persist-credentials: false")
    return i + 2


def _process_checkout_without_block(indent, new_lines, i):
    """Process a checkout step that needs a 'with:' block added."""
    new_lines.append(f"{indent}  with:")
    new_lines.append(f"{indent}    persist-credentials: false")
    return i + 1


def _handle_checkout_step(lines, i, line, skip_indices, checkout_idx, new_lines):
    indent = _extract_indent(line)
    new_lines.append(line)

    if _should_add_credentials(checkout_idx, skip_indices):
        if i + 1 < len(lines) and lines[i + 1].strip().startswith("with:"):
            return _process_checkout_with_block(lines, i, indent, new_lines)
        else:
            return _process_checkout_without_block(indent, new_lines, i)
    else:
        return i + 1


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
            i = _handle_checkout_step(
                lines, i, line, skip_indices, checkout_idx, new_lines
            )
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
