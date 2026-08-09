"""Core data models for qlty analysis."""

from dataclasses import dataclass, field
from enum import IntEnum
from typing import Any


class Severity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    INFO = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> "Severity":
        mapping = {
            "error": cls.ERROR,
            "warning": cls.WARNING,
            "note": cls.INFO,
        }
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        return {
            self.ERROR: "🔴",
            self.WARNING: "🟠",
            self.INFO: "🔵",
            self.NONE: "⚪",
        }[self]


@dataclass
class SarifIssue:
    """Unified representation of a SARIF result (check or smell)."""

    rule_id: str
    level: Severity
    message: str
    file_path: str
    start_line: int
    end_line: int | None = None
    category: str = ""  # check, smell, security, format, etc.
    help_uri: str = ""
    metadata: dict[str, Any] = field(default_factory=dict)
    fingerprints: dict[str, str] = field(default_factory=dict)

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
