import json

data = json.load(open("/home/marlonsc/mcb-check-fix/qlty.check.lst"))
c = {}
for run in data.get("runs", []):
    for r in run.get("results", []):
        rid = r["ruleId"]
        c[rid] = c.get(rid, 0) + 1
for k, v in sorted(c.items(), key=lambda x: -x[1]):
    print(f"{v:3d} {k}")
