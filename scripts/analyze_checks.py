#!/usr/bin/env python3
"""Analyze qlty check results from SARIF format."""

import json
import sys
from collections import defaultdict


def load_sarif(filepath):
    """Load and validate SARIF data from a file."""
    try:
        with open(filepath) as f:
            content = f.read().strip()
    except FileNotFoundError:
        print("ERROR: " + filepath + " not found")
        print("Run 'qlty check --all --sarif > " + filepath + "' first")
        sys.exit(1)

    if not content:
        print("ERROR: " + filepath + " is empty")
        sys.exit(1)

    try:
        return json.loads(content)
    except json.JSONDecodeError as e:
        print("ERROR: Invalid JSON: " + str(e))
        sys.exit(1)


def extract_issue(result):
    """Extract path, line, and message from a SARIF result."""
    loc = result.get("locations", [{}])[0].get("physicalLocation", {})
    path = loc.get("artifactLocation", {}).get("uri", "?")
    region = loc.get("region", {})
    line = region.get("startLine", "?")
    msg = result.get("message", {}).get("text", "")
    return path, line, msg


def main():
    """Analyze qlty check results and print summary."""
    data = load_sarif("qlty.check.lst")
    results = data["runs"][0].get("results", [])

    if not results:
        print("No issues found!")
        return

    groups = defaultdict(list)
    fixture_count = 0
    real_count = 0

    for result in results:
        rule = result["ruleId"]
        path, line, msg = extract_issue(result)

        if "fixtures/" in path:
            fixture_count += 1
        else:
            real_count += 1

        groups[rule].append({"path": path, "line": line, "msg": msg})

    # Print summary
    print("\n" + "=" * 70)
    print("QLTY CHECK SUMMARY")
    print("=" * 70)
    print("Total issues: " + str(len(results)))
    print("  In fixtures (should be excluded): " + str(fixture_count))
    print("  In real code: " + str(real_count))
    print()

    for rule in sorted(groups.keys(), key=lambda r: -len(groups[r])):
        items = groups[rule]
        print("\n" + rule + ": " + str(len(items)) + " issues")
        print("-" * 70)
        for item in items[:10]:
            print("  " + item["path"] + ":" + str(item["line"]))
            if item["msg"]:
                print("    -> " + item["msg"][:100])
        if len(items) > 10:
            print("  ... and " + str(len(items) - 10) + " more")


if __name__ == "__main__":
    main()
