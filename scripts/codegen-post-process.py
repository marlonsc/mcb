#!/usr/bin/env python3
"""Post-process generated mod.rs to add singular module aliases.

sea-orm-cli generates plural module names (matching table names).
Existing code uses singular names. This script appends `pub use X as Y;`
aliases so both forms resolve.
"""

import re
import sys

IRREGULAR_PLURALS = {
    "branches": "branch",
    "repositories": "repository",
    "worktrees": "worktree",
    "entries": "entry",
    "indices": "index",
}


def singularize(plural: str) -> str:
    if plural in IRREGULAR_PLURALS:
        return IRREGULAR_PLURALS[plural]
    if plural.endswith("sses") or plural.endswith("shes") or plural.endswith("ches"):
        return plural[:-2]
    if plural.endswith("ies"):
        return plural[:-3] + "y"
    if plural.endswith("s") and not plural.endswith("ss"):
        return plural[:-1]
    return plural


def main():
    if len(sys.argv) < 2:
        print("Usage: codegen-post-process.py <mod.rs>", file=sys.stderr)
        sys.exit(1)

    mod_path = sys.argv[1]
    with open(mod_path) as f:
        content = f.read()

    modules = re.findall(r"pub mod (\w+);", content)
    modules = [m for m in modules if m != "prelude"]

    aliases = []
    for mod_name in modules:
        singular = singularize(mod_name)
        if singular != mod_name:
            aliases.append(f"pub use {mod_name} as {singular};")

    if aliases:
        content = content.rstrip() + "\n\n" + "\n".join(aliases) + "\n"

    with open(mod_path, "w") as f:
        f.write(content)


if __name__ == "__main__":
    main()
