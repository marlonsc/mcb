#!/usr/bin/env python3
"""Analyze qlty.smells.lst from SARIF format."""

import json
from collections import defaultdict

with open("qlty.smells.lst") as f:
    data = json.load(f)

results = data["runs"][0]["results"]

# Group by ruleId and file
by_rule = defaultdict(list)
by_file = defaultdict(list)
rule_counts: dict[str, int] = defaultdict(int)

for r in results:
    rule = r["ruleId"]
    uri = r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
    msg = r["message"]["text"]
    start = r["locations"][0]["physicalLocation"]["region"]["startLine"]
    end = r["locations"][0]["physicalLocation"]["region"]["endLine"]

    entry = {"rule": rule, "file": uri, "msg": msg, "start": start, "end": end}
    by_rule[rule].append(entry)
    by_file[uri].append(entry)
    rule_counts[rule] += 1

print("=== TOTAL RESULTS:", len(results))
print()
print("=== BY RULE ===")
for rule, count in sorted(rule_counts.items(), key=lambda x: -x[1]):
    print(f"  {rule}: {count}")
print()

# filter out test fixtures
real_results = [
    r
    for r in results
    if "fixtures"
    not in r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
]
print("=== REAL RESULTS (excluding fixtures):", len(real_results))
real_by_rule: dict[str, int] = defaultdict(int)
for r in real_results:
    real_by_rule[r["ruleId"]] += 1
for rule, count in sorted(real_by_rule.items(), key=lambda x: -x[1]):
    print(f"  {rule}: {count}")
print()

print("=== REAL FILES AFFECTED ===")
real_files = set()
for r in real_results:
    real_files.add(r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"])
for f in sorted(real_files):
    print(f"  {f}")
print()

print("=== DETAIL BY RULE (excluding fixtures) ===")
for rule in sorted(real_by_rule.keys()):
    print(f"\n--- {rule} ---")
    for r in real_results:
        if r["ruleId"] == rule:
            uri = r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
            msg = r["message"]["text"]
            start = r["locations"][0]["physicalLocation"]["region"]["startLine"]
            end = r["locations"][0]["physicalLocation"]["region"]["endLine"]
            print(f"  {uri}:{start}-{end} | {msg}")
