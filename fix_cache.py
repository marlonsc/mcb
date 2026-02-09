#!/usr/bin/env python3
"""Add save-if to Swatinem/rust-cache steps."""

import re


def add_save_if(filepath):  # noqa: C901
    with open(filepath) as f:
        lines = f.readlines()

    new_lines = []
    i = 0
    fixed = 0

    while i < len(lines):
        line = lines[i]

        if "Swatinem/rust-cache@" in line:
            indent_match = re.match(r"^(\s*)", line)
            indent = indent_match.group(1) if indent_match else "      "

            new_lines.append(line)

            # Check if next line has 'with:'
            if i + 1 < len(lines) and lines[i + 1].strip().startswith("with:"):
                new_lines.append(lines[i + 1])
                i += 2
                # Check if save-if already exists
                has_save_if = False
                while (
                    i < len(lines)
                    and lines[i].strip()
                    and re.match(r"^" + indent + r"  \S", lines[i])
                ):
                    if "save-if" in lines[i]:
                        has_save_if = True
                    new_lines.append(lines[i])
                    i += 1
                if not has_save_if:
                    new_lines.insert(
                        -1 if new_lines else 0,
                        f"{indent}  save-if: ${{{{ github.event_name == 'push' }}}}\n",
                    )
                    fixed += 1
            else:
                # No 'with:' block - add one
                new_lines.append(f"{indent}  with:\n")
                new_lines.append(
                    f"{indent}    save-if: ${{{{ github.event_name == 'push' }}}}\n"
                )
                fixed += 1
                i += 1
        else:
            new_lines.append(line)
            i += 1

    with open(filepath, "w") as f:
        f.writelines(new_lines)

    print(f"Fixed {filepath}: added save-if to {fixed} cache steps")


add_save_if("/home/marlonsc/mcb-check-fix/.github/workflows/docs.yml")
add_save_if("/home/marlonsc/mcb-check-fix/.github/workflows/release.yml")
