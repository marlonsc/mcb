"""Canonical MCB constant declarations — the single definition home (ENFORCE-079).

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from enum import IntEnum, StrEnum
from typing import Final

from flext_core import FlextConstants


class McbScriptsSeverity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    INFO = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> McbScriptsSeverity:
        """Parse a SARIF severity token into a severity.

        Returns:
            The resulting ``McbScriptsSeverity``.
        """
        mapping = {"error": cls.ERROR, "warning": cls.WARNING, "note": cls.INFO}
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        """Return the display glyph for this severity.

        Returns:
            The resulting ``str``.
        """
        return {self.ERROR: "🔴", self.WARNING: "🟠", self.INFO: "🔵", self.NONE: "⚪"}[
            self
        ]


class McbScriptsQltyCategory(StrEnum):
    """Quality-report categories (SSOT for the closed vocabulary).

    ``check`` and ``smell`` come from the qlty runners; ``security``,
    ``format`` and the rest name the remaining report buckets.
    """

    CHECK = "check"
    SMELL = "smell"
    SECURITY = "security"
    FORMAT = "format"


class McbScriptsConstants(FlextConstants):
    """Project-wide constants for MCB Python automation.

    MRO composition over the fleet ``FlextConstants`` keeps the constants
    namespace fleet-consistent; the names below are MCB-specific additions
    and override no ``Final`` declaration of the inherited family topics
    (``ENV_FILE_DEFAULT``/``DEFAULT_ENCODING`` reuse the identical fleet
    values through the MRO). Canonical FLEXT constants remain available
    through ``flext_core.c``.
    """

    # -- Environment --
    ENV_PREFIX: Final[str] = "MCB_"
    ENV_FILE_ENV_KEY: Final[str] = "MCB_ENV_FILE"

    # -- Project identity --
    PROJECT_NAME: Final[str] = "mcb"


__all__ = [
    "McbScriptsConstants",
    "McbScriptsQltyCategory",
    "McbScriptsSeverity",
]
