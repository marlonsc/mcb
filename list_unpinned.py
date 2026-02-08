import json

data = json.load(open("/home/marlonsc/mcb-check-fix/qlty.check.lst"))
for run in data.get("runs", []):
    for r in run.get("results", []):
        rid = r["ruleId"]
        if "unpinned" in rid:
            loc = r.get("locations", [{}])[0].get("physicalLocation", {})
            uri = loc.get("artifactLocation", {}).get("uri", "?")
            region = loc.get("region", {})
            line = region.get("startLine", "?")
            msg = r.get("message", {}).get("text", "")[:100]
            print(f"{uri}:{line}  {msg}")
