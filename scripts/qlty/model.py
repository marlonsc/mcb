"""Qlty Model.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from enum import IntEnum

from pydantic import BaseModel, ConfigDict, Field, JsonValue


class Severity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    INFO = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> Severity:
        mapping = {"error": cls.ERROR, "warning": cls.WARNING, "note": cls.INFO}
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        return {self.ERROR: "🔴", self.WARNING: "🟠", self.INFO: "🔵", self.NONE: "⚪"}[
            self
        ]


class SarifArtifactLocation(BaseModel):
    """SARIF artifactLocation object."""

    model_config = ConfigDict(populate_by_name=True)

    uri: str = Field(default="unknown", alias="uri")


class SarifRegion(BaseModel):
    """SARIF region object."""

    model_config = ConfigDict(populate_by_name=True)

    start_line: int = Field(default=0, alias="startLine")
    end_line: int | None = Field(default=None, alias="endLine")


class SarifPhysicalLocation(BaseModel):
    """SARIF physicalLocation object."""

    model_config = ConfigDict(populate_by_name=True)

    artifact_location: SarifArtifactLocation = Field(
        default_factory=SarifArtifactLocation, alias="artifactLocation"
    )
    region: SarifRegion | None = Field(default=None, alias="region")


class SarifLocation(BaseModel):
    """SARIF location object."""

    model_config = ConfigDict(populate_by_name=True)

    physical_location: SarifPhysicalLocation | None = Field(
        default=None, alias="physicalLocation"
    )


class SarifMessage(BaseModel):
    """SARIF message object."""

    model_config = ConfigDict(populate_by_name=True)

    text: str = Field(default="", alias="text")


class SarifRun(BaseModel):
    """SARIF run object."""

    model_config = ConfigDict(populate_by_name=True)

    results: list[SarifResult] = Field(default_factory=list, alias="results")


class SarifResult(BaseModel):
    """SARIF result object."""

    model_config = ConfigDict(populate_by_name=True)

    rule_id: str = Field(default="unknown", alias="ruleId")
    level: str = Field(default="note", alias="level")
    message: SarifMessage = Field(default_factory=SarifMessage, alias="message")
    locations: list[SarifLocation] = Field(default_factory=list, alias="locations")
    properties: dict[str, JsonValue] = Field(default_factory=dict, alias="properties")
    partial_fingerprints: dict[str, str] = Field(
        default_factory=dict, alias="partialFingerprints"
    )
    fingerprints: dict[str, str] = Field(default_factory=dict, alias="fingerprints")


class SarifIssue(BaseModel):
    """Unified representation of a SARIF result (check or smell)."""

    model_config = ConfigDict(populate_by_name=True, extra="ignore")

    rule_id: str
    level: Severity
    message: str
    file_path: str
    start_line: int
    end_line: int | None = None
    category: str = ""  # check, smell, security, format, etc.
    help_uri: str = ""
    metadata: dict[str, JsonValue] = {}
    fingerprints: dict[str, str] = {}

    @property
    def location_str(self) -> str:
        if self.end_line and self.end_line != self.start_line:
            return f"{self.file_path}:{self.start_line}-{self.end_line}"
        return f"{self.file_path}:{self.start_line}"

    @property
    def rule_category(self) -> str:
        """Extract category from rule_id (e.g., 'rustfmt', 'zizmor', 'osv-scanner')."""
        if ":" in self.rule_id:
            return self.rule_id.split(":")[0]
        return "unknown"
