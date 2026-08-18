"""Qlty Parser.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import json
from pathlib import Path

from mcb_scripts.core import get_logger, r

from mcb_scripts.qlty.model import SarifIssue, SarifResult, SarifRun, Severity

logger = get_logger(__name__)


def _result_to_issue(result: SarifResult) -> SarifIssue | None:
    """Convert a single SARIF result into a normalized issue."""
    locations = result.locations
    if not locations:
        return None

    physical = locations[0].physical_location
    if physical is None:
        return None

    region = physical.region
    start_line = region.start_line if region else 0
    end_line = region.end_line if region else None

    fingerprints = result.partial_fingerprints or result.fingerprints

    return SarifIssue(
        rule_id=result.rule_id,
        level=Severity.from_str(result.level),
        message=result.message.text,
        file_path=physical.artifact_location.uri,
        start_line=start_line,
        end_line=end_line,
        metadata=result.properties,
        fingerprints=fingerprints,
    )


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

    run = SarifRun.model_validate(data)

    issues: list[SarifIssue] = []
    for result in run.results:
        issue = _result_to_issue(result)
        if issue is not None:
            issues.append(issue)

    return r[list[SarifIssue]].ok(issues)
