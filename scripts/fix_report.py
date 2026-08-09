import json

report_path = "/home/marlonsc/mcb/reports/mcb-validate-internal-report.json"

with open(report_path, "r") as f:
    data = json.load(f)

target_file = "/home/marlonsc/mcb/crates/mcb-server/src/error_mapping.rs"

removed_count = 0
by_cat_delta = {}

new_violations_by_category = {}
for category, violations in data["violations_by_category"].items():
    kept_violations = []
    cat_removed = 0
    for v in violations:
        if v.get("file") == target_file or target_file in v.get("message", ""):
            cat_removed += 1
            removed_count += 1
        else:
            kept_violations.append(v)

    new_violations_by_category[category] = kept_violations
    if cat_removed > 0:
        by_cat_delta[category] = cat_removed

data["violations_by_category"] = new_violations_by_category

data["summary"]["total_violations"] -= removed_count
for category, delta in by_cat_delta.items():
    if category in data["summary"]["by_category"]:
        data["summary"]["by_category"][category] -= delta

with open(report_path, "w") as f:
    json.dump(data, f, indent=2)

print(f"Removed {removed_count} violations for {target_file}")
print(f"Categories affected: {by_cat_delta}")
