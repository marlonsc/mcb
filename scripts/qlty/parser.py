"""Qlty Parser.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import json
from pathlib import Path

from lib.core import get_logger, r

from qlty.model import SarifIssue, Severity

logger = get_logger(__name__)


def parse_sarif_file(path: Path) -> r[list[SarifIssue]]:
    """Parse SARIF JSON and extract all issues."""
    try:
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        return r[list[SarifIssue]].fail(f"SARIF file not found: {path}")
    except json.JSONDecodeError as exc:
        return r[list[SarifIssue]].fail(f"invalid SARIF JSON in {path}: {exc}")
    except OSError as exc:
        return r[list[SarifIssue]].fail(f"cannot read {path}: {exc}")

    issues: list[SarifIssue] = []
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

    return r[list[SarifIssue]].ok(issues)
