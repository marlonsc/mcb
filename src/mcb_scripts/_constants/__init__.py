"""Canonical MCB constants — the single declaration home (ENFORCE-079).

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from enum import IntEnum, StrEnum
from typing import Final


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


class McbScriptsConstants:
    """Project-wide constants for MCB Python automation.

    This is a standalone SSOT class rather than a subclass of
    :class:`flext_core.FlextConstants` because the FLEXT constants declare
    environment names as ``Final`` and therefore cannot be overridden in a
    subclass without breaking static typing. MCB-specific values are kept here;
    canonical FLEXT constants remain available through ``flext_core.c``.
    """

    # -- Environment / encoding --
    ENV_PREFIX: Final[str] = "MCB_"
    ENV_FILE_ENV_VAR: Final[str] = "MCB_ENV_FILE"
    ENV_FILE_DEFAULT: Final[str] = ".env"
    DEFAULT_ENCODING: Final[str] = "utf-8"

    # -- Project identity --
    PROJECT_NAME: Final[str] = "mcb"


__all__ = [
    "McbScriptsConstants",
    "McbScriptsQltyCategory",
    "McbScriptsSeverity",
]
