#!/usr/bin/env python3
"""Extract actionable issues from qlty.check.lst (excludes fixtures and Cargo.lock)."""

import json
import sys
from collections import defaultdict


def main():
    try:
        with open("qlty.check.lst") as f:
            data = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"ERROR: {e}")
        sys.exit(1)

    results = data["runs"][0].get("results", [])
    groups = defaultdict(list)

    for r in results:
        rule = r["ruleId"]
        loc = r.get("locations", [{}])[0].get("physicalLocation", {})
        path = loc.get("artifactLocation", {}).get("uri", "?")
        region = loc.get("region", {})
        line = region.get("startLine", "?")
        msg = r.get("message", {}).get("text", "")

        # Skip fixtures and Cargo.lock (not actionable)
        if "fixtures/" in path or "Cargo.lock" in path:
            continue

        groups[rule].append({"path": path, "line": line, "msg": msg})

    total = sum(len(v) for v in groups.values())
    print(f"Actionable issues: {total}")
    print()

    for rule in sorted(groups.keys(), key=lambda r: -len(groups[r])):
        items = groups[rule]
        print(f"{rule}: {len(items)} issues")
        for item in items:
            print(f"  {item['path']}:{item['line']}  {item['msg'][:120]}")
        print()


if __name__ == "__main__":
    main()
