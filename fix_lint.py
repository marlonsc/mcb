#!/usr/bin/env python3
"""Find lines over 79 chars."""


path = "/home/marlonsc/mcb-smells-fix/scripts/fix_smells.py"
with open(path) as f:
    lines = f.readlines()

for i, line in enumerate(lines, 1):
    stripped = line.rstrip("\n")
    if len(stripped) > 79:
        print(f"L{i} ({len(stripped)}): {stripped}")
