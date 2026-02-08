#!/usr/bin/env python3
import json

with open("qlty.check.lst") as f:
    data = json.load(f)

results = data["runs"][0].get("results", [])

# Group by ruleId
from collections import defaultdict

groups = defaultdict(list)
for r in results:
    rule = r["ruleId"]
    loc = r.get("locations", [{}])[0].get("physicalLocation", {})
    path = loc.get("artifactLocation", {}).get("uri", "?")
    region = loc.get("region", {})
    line = region.get("startLine", "?")
    msg = r.get("message", {}).get("text", "")
    groups[rule].append({"path": path, "line": line, "msg": msg})

# Print summary
for rule in sorted(groups.keys()):
    items = groups[rule]
    print(f"\n{'=' * 60}")
    print(f"RULE: {rule} ({len(items)} issues)")
    print(f"{'=' * 60}")
    for item in items:
        print(f"  {item['path']}:{item['line']}  {item['msg'][:120]}")
