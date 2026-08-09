"""SARIF parsing logic."""

import json
from pathlib import Path

from qlty.model import SarifIssue, Severity


def parse_sarif_file(path: Path) -> list[SarifIssue]:
    """Parse SARIF JSON and extract all issues."""
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    issues = []
    for run in data.get("runs", []):
        results = run.get("results", [])
        for result in results:
            rule_id = result.get("ruleId", "unknown")
            level_str = result.get("level", "note")
            level = Severity.from_str(level_str)
            message = result.get("message", {}).get("text", "")

            # Extract location
            locations = result.get("locations", [])
            if not locations:
                continue

            physical_loc = locations[0].get("physicalLocation", {})
            artifact_loc = physical_loc.get("artifactLocation", {})
            file_path = artifact_loc.get("uri", "unknown")

            region = physical_loc.get("region", {})
            start_line = region.get("startLine", 0)
            end_line = region.get("endLine", start_line)

            # Extract metadata and fingerprints
            metadata = {}
            if "properties" in result:
                metadata = result["properties"]

            fingerprints = result.get("partialFingerprints", {})
            if not fingerprints:
                fingerprints = result.get("fingerprints", {})

            issues.append(
                SarifIssue(
                    rule_id=rule_id,
                    level=level,
                    message=message,
                    file_path=file_path,
                    start_line=start_line,
                    end_line=end_line,
                    metadata=metadata,
                    fingerprints=fingerprints,
                )
            )

    return issues
