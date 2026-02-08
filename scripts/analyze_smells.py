#!/usr/bin/env python3
"""Analyze qlty smells results from SARIF format."""

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
        print("Run 'qlty smells --all --sarif > " + filepath + "' first")
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


def print_group(rule, items, report_file=None):
    """Print and optionally write a group of issues."""
    header = "\n" + rule + ": " + str(len(items)) + " issues"
    separator = "-" * 70
    print(header)
    print(separator)
    if report_file:
        report_file.write(header + "\n")
        report_file.write(separator + "\n")

    for item in items[:10]:
        line_info = "  " + item["path"] + ":" + str(item["line"])
        print(line_info)
        if report_file:
            report_file.write(line_info + "\n")
        if item["msg"]:
            msg_info = "    -> " + item["msg"][:100]
            print(msg_info)
            if report_file:
                report_file.write(msg_info + "\n")

    if len(items) > 10:
        remaining = "  ... and " + str(len(items) - 10) + " more"
        print(remaining)
        if report_file:
            report_file.write(remaining + "\n")


def main():
    """Analyze qlty smells results and print summary."""
    data = load_sarif("qlty.smells.lst")
    results = data["runs"][0].get("results", [])

    if not results:
        print("No code smells found!")
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
    print("QLTY SMELLS SUMMARY")
    print("=" * 70)
    print("Total smells: " + str(len(results)))
    print("  In fixtures (should be excluded): " + str(fixture_count))
    print("  In real code: " + str(real_count))
    print()

    # Save detailed report
    with open("qlty_smells_report.txt", "w") as report:
        report.write("QLTY SMELLS REPORT\n")
        report.write("=" * 70 + "\n")
        report.write("Total smells: " + str(len(results)) + "\n")
        report.write("  In fixtures: " + str(fixture_count) + "\n")
        report.write("  In real code: " + str(real_count) + "\n\n")

        for rule in sorted(groups.keys(), key=lambda r: -len(groups[r])):
            print_group(rule, groups[rule], report)

    print("\nDetailed report saved to: qlty_smells_report.txt")


if __name__ == "__main__":
    main()
