"""FLEXT-style structured logging for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import cast

from flext_core import FlextUtilitiesLogging


class McbLogger(FlextUtilitiesLogging):
    """Structured logger facade wired to MCB defaults."""

    @classmethod
    def fetch_logger(cls, name: str | None = None) -> McbLogger:
        """Fetch the canonical logger for a module as an ``McbLogger``."""
        resolved_name = name or "__main__"
        base = cast(FlextUtilitiesLogging, super().fetch_logger(resolved_name))
        return cls(resolved_name, _bound_logger=base.logger)


def configure_logging(json_format: bool = False) -> None:
    """Configure structlog for MCB scripts.

    Call once at each CLI entrypoint. In normal dev mode logs are rendered as
    plain key=value lines; set ``json_format=True`` (or ``MCB_LOG_JSON=1``) for
    JSON output suitable for CI aggregation.
    """
    McbLogger.configure_structlog(console_renderer=not json_format)


def get_logger(name: str) -> McbLogger:
    """Fetch a structured logger for the given module name."""
    return McbLogger.fetch_logger(name)


__all__ = ["McbLogger", "configure_logging", "get_logger"]
