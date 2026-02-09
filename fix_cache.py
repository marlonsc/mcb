#!/usr/bin/env python3
"""Add save-if to Swatinem/rust-cache steps."""

import re


def _extract_indent_from_line(line):
    indent_match = re.match(r"^(\s*)", line)
    return indent_match.group(1) if indent_match else "      "


def _check_existing_save_if(lines, i, indent):
    has_save_if = False
    start_idx = i
    while (
        i < len(lines)
        and lines[i].strip()
        and re.match(r"^" + indent + r"  \S", lines[i])
    ):
        if "save-if" in lines[i]:
            has_save_if = True
        i += 1
    return has_save_if, i - start_idx


def _process_cache_with_block(lines, i, indent, new_lines):
    new_lines.append(lines[i + 1])
    has_save_if, consumed_lines = _check_existing_save_if(lines, i + 2, indent)

    for j in range(consumed_lines):
        new_lines.append(lines[i + 2 + j])

    fixed = 0
    if not has_save_if:
        new_lines.insert(
            -1 if new_lines else 0,
            f"{indent}  save-if: ${{{{ github.event_name == 'push' }}}}\n",
        )
        fixed = 1

    return i + 2 + consumed_lines, fixed


def _process_cache_without_block(indent, new_lines, i):
    new_lines.append(f"{indent}  with:\n")
    new_lines.append(f"{indent}    save-if: ${{{{ github.event_name == 'push' }}}}\n")
    return i + 1, 1


def add_save_if(filepath):
    with open(filepath) as f:
        lines = f.readlines()

    new_lines = []
    i = 0
    fixed = 0

    while i < len(lines):
        line = lines[i]

        if "Swatinem/rust-cache@" in line:
            indent = _extract_indent_from_line(line)
            new_lines.append(line)

            if i + 1 < len(lines) and lines[i + 1].strip().startswith("with:"):
                i, fixes = _process_cache_with_block(lines, i, indent, new_lines)
                fixed += fixes
            else:
                i, fixes = _process_cache_without_block(indent, new_lines, i)
                fixed += fixes
        else:
            new_lines.append(line)
            i += 1

    with open(filepath, "w") as f:
        f.writelines(new_lines)

    print(f"Fixed {filepath}: added save-if to {fixed} cache steps")


add_save_if("/home/marlonsc/mcb-check-fix/.github/workflows/docs.yml")
add_save_if("/home/marlonsc/mcb-check-fix/.github/workflows/release.yml")
