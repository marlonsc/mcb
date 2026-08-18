"""FLEXT-style structured logging for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_core import FlextUtilitiesLogging, p, u


def configure_logging(*, json_format: bool = False) -> None:
    """Configure structlog for MCB scripts.

    Call once at each CLI entrypoint. In normal dev mode logs are rendered as
    plain key=value lines; set ``json_format=True`` (or ``MCB_LOG_JSON=1``) for
    JSON output suitable for CI aggregation.
    """
    FlextUtilitiesLogging.configure_structlog(console_renderer=not json_format)


def get_logger(name: str) -> p.Logger:
    """Fetch a structured logger for the given module name."""
    return u.fetch_logger(name)


__all__ = ["configure_logging", "get_logger"]
