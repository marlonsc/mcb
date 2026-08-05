"""FLEXT-style core kernel for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT

This module re-exports the canonical FLEXT primitives wired for MCB:

- ``McbResult[T]`` (alias ``r``) — explicit fallible results.
- ``BaseMcbSettings`` — Pydantic ``BaseSettings`` with singleton lifecycle.
- ``McbService`` (alias ``s``) — per-class singleton service base.
- ``McbLogger`` / ``get_logger`` / ``configure_logging`` — structured logging.

Concrete implementations live in sibling modules so each module owns one
public abstraction (``result``, ``settings``, ``logger``, ``service``).
"""

from __future__ import annotations

from .cli import create_app_with_common_params, register_result_command
from .constants import McbConstants, c
from .logger import McbLogger, configure_logging, get_logger
from .result import McbResult, r
from .service import McbScriptsService, McbService, s
from .settings import BaseCommandSettings, BaseMcbSettings

__all__ = [
    "BaseCommandSettings",
    "BaseMcbSettings",
    "McbConstants",
    "McbLogger",
    "McbResult",
    "McbScriptsService",
    "McbService",
    "c",
    "configure_logging",
    "create_app_with_common_params",
    "get_logger",
    "r",
    "register_result_command",
    "s",
]
