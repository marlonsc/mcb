"""Qlty Model.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from collections.abc import Mapping
from types import MappingProxyType

from flext_core import m, t
from mcb_scripts._constants import McbScriptsQltyCategory, McbScriptsSeverity

__all__ = ["McbScriptsQltyCategory", "McbScriptsSeverity"]


def _empty_str_mapping() -> Mapping[str, str]:
    """Return an immutable empty string mapping.

    Returns:
        The resulting ``Mapping[str, str]``.
    """
    return MappingProxyType({})


def _empty_json_mapping() -> t.JsonMapping:
    """Return an immutable empty JSON mapping.

    Returns:
        The resulting ``t.JsonMapping``.
    """
    return MappingProxyType({})


class McbScriptsSarifArtifactLocation(m.BaseModel):
    """SARIF artifactLocation object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    uri: str = m.Field(default="unknown", alias="uri", description="Artifact URI")


class McbScriptsSarifRegion(m.BaseModel):
    """SARIF region object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    start_line: int = m.Field(
        default=0, alias="startLine", description="First line of the region",
    )
    end_line: int | None = m.Field(
        default=None, alias="endLine", description="Last line of the region",
    )


class McbScriptsSarifPhysicalLocation(m.BaseModel):
    """SARIF physicalLocation object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    artifact_location: McbScriptsSarifArtifactLocation = m.Field(
        default_factory=McbScriptsSarifArtifactLocation,
        alias="artifactLocation",
        description="Artifact referenced by this location",
    )
    region: McbScriptsSarifRegion | None = m.Field(
        default=None, alias="region", description="Region inside the artifact",
    )


class McbScriptsSarifLocation(m.BaseModel):
    """SARIF location object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    physical_location: McbScriptsSarifPhysicalLocation | None = m.Field(
        default=None,
        alias="physicalLocation",
        description="Physical location of the finding",
    )


class McbScriptsSarifMessage(m.BaseModel):
    """SARIF message object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    text: str = m.Field(default="", alias="text", description="Message text")


class McbScriptsSarifRun(m.BaseModel):
    """SARIF run object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    results: tuple[McbScriptsSarifResult, ...] = m.Field(
        default=(), alias="results", description="Results in this run",
    )


class McbScriptsSarifResult(m.BaseModel):
    """SARIF result object."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    rule_id: str = m.Field(
        default="unknown", alias="ruleId", description="Identifier of the rule",
    )
    level: str = m.Field(
        default="note", alias="level", description="SARIF severity level token",
    )
    message: McbScriptsSarifMessage = m.Field(
        default_factory=McbScriptsSarifMessage,
        alias="message",
        description="Human-readable message",
    )
    locations: tuple[McbScriptsSarifLocation, ...] = m.Field(
        default=(), alias="locations", description="Locations of the finding",
    )
    properties: t.JsonMapping = m.Field(
        default_factory=_empty_json_mapping,
        alias="properties",
        description="Tool properties",
    )
    partial_fingerprints: Mapping[str, str] = m.Field(
        default_factory=_empty_str_mapping,
        alias="partialFingerprints",
        description="Partial fingerprints of the finding",
    )
    fingerprints: Mapping[str, str] = m.Field(
        default_factory=_empty_str_mapping,
        description="Stable fingerprints of the finding",
    )


class McbScriptsSarifIssue(m.BaseModel):
    """Unified representation of a SARIF result (check or smell)."""

    model_config = m.ConfigDict(populate_by_name=True, extra="forbid")

    rule_id: str = m.Field(description="Identifier of the rule")
    level: McbScriptsSeverity = m.Field(description="Normalized severity")
    message: str = m.Field(description="Human-readable message")
    file_path: str = m.Field(description="File the finding refers to")
    start_line: int = m.Field(description="First line of the finding")
    end_line: int | None = m.Field(
        default=None, description="Last line of the finding",
    )
    # Open vocabulary: external tools name their own categories (gitops,
    # rustfmt, zizmor, ...). McbScriptsQltyCategory is the SSOT only for the values OUR
    # runners assign (check/smell); never turn this field into a closed enum.
    category: str = m.Field(default="", description="Report bucket of the finding")
    help_uri: str = m.Field(default="", description="Documentation URI for the rule")
    metadata: t.JsonMapping = m.Field(
        default_factory=_empty_json_mapping,
        description="Tool-provided metadata",
    )
    fingerprints: Mapping[str, str] = m.Field(
        default_factory=_empty_str_mapping,
        description="Stable fingerprints of the finding",
    )

    @property
    def location_str(self) -> str:
        """Format this issue's file and line as a single location string."""
        if self.end_line and self.end_line != self.start_line:
            return f"{self.file_path}:{self.start_line}-{self.end_line}"
        return f"{self.file_path}:{self.start_line}"

    @property
    def rule_category(self) -> str:
        """Extract category from rule_id (e.g., 'rustfmt', 'zizmor', 'osv-scanner')."""
        if ":" in self.rule_id:
            return self.rule_id.split(":")[0]
        return "unknown"
