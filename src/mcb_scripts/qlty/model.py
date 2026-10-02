"""Qlty Model.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from enum import IntEnum, StrEnum

from flext_core import m, t


class QltyCategory(StrEnum):
    """Quality-report categories (SSOT for the closed vocabulary).

    ``check`` and ``smell`` come from the qlty runners; ``security``,
    ``format`` and the rest name the remaining report buckets.
    """

    CHECK = "check"
    SMELL = "smell"
    SECURITY = "security"
    FORMAT = "format"


class Severity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    INFO = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> Severity:
        """Parse a SARIF severity token into a Severity.

        Returns:
            The resulting ``Severity``.
        """
        mapping = {"error": cls.ERROR, "warning": cls.WARNING, "note": cls.INFO}
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        """Return the display glyph for this severity."""
        return {self.ERROR: "🔴", self.WARNING: "🟠", self.INFO: "🔵", self.NONE: "⚪"}[
            self
        ]


class SarifArtifactLocation(m.BaseModel):
    """SARIF artifactLocation object."""

    model_config = m.ConfigDict(populate_by_name=True)

    uri: str = m.Field(default="unknown", alias="uri")


class SarifRegion(m.BaseModel):
    """SARIF region object."""

    model_config = m.ConfigDict(populate_by_name=True)

    start_line: int = m.Field(default=0, alias="startLine")
    end_line: int | None = m.Field(default=None, alias="endLine")


class SarifPhysicalLocation(m.BaseModel):
    """SARIF physicalLocation object."""

    model_config = m.ConfigDict(populate_by_name=True)

    artifact_location: SarifArtifactLocation = m.Field(
        default_factory=SarifArtifactLocation, alias="artifactLocation",
    )
    region: SarifRegion | None = m.Field(default=None, alias="region")


class SarifLocation(m.BaseModel):
    """SARIF location object."""

    model_config = m.ConfigDict(populate_by_name=True)

    physical_location: SarifPhysicalLocation | None = m.Field(
        default=None, alias="physicalLocation",
    )


class SarifMessage(m.BaseModel):
    """SARIF message object."""

    model_config = m.ConfigDict(populate_by_name=True)

    text: str = m.Field(default="", alias="text")


class SarifRun(m.BaseModel):
    """SARIF run object."""

    model_config = m.ConfigDict(populate_by_name=True)

    results: list[SarifResult] = m.Field(default_factory=list, alias="results")


class SarifResult(m.BaseModel):
    """SARIF result object."""

    model_config = m.ConfigDict(populate_by_name=True)

    rule_id: str = m.Field(default="unknown", alias="ruleId")
    level: str = m.Field(default="note", alias="level")
    message: SarifMessage = m.Field(default_factory=SarifMessage, alias="message")
    locations: list[SarifLocation] = m.Field(default_factory=list, alias="locations")
    properties: dict[str, t.JsonValue] = m.Field(
        default_factory=dict, alias="properties",
    )
    partial_fingerprints: dict[str, str] = m.Field(
        default_factory=dict, alias="partialFingerprints",
    )
    fingerprints: dict[str, str] = m.Field(default_factory=dict, alias="fingerprints")


class SarifIssue(m.BaseModel):
    """Unified representation of a SARIF result (check or smell)."""

    model_config = m.ConfigDict(populate_by_name=True, extra="ignore")

    rule_id: str
    level: Severity
    message: str
    file_path: str
    start_line: int
    end_line: int | None = None
    # Open vocabulary: external tools name their own categories (gitops,
    # rustfmt, zizmor, ...). QltyCategory is the SSOT only for the values OUR
    # runners assign (check/smell); never turn this field into a closed enum.
    category: str = ""
    help_uri: str = ""
    metadata: dict[str, t.JsonValue] = m.Field(default_factory=dict)
    fingerprints: dict[str, str] = m.Field(default_factory=dict)

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
