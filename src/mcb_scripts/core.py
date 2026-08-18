"""FLEXT-style core kernel for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT

This module re-exports the canonical FLEXT primitives wired for MCB:

- ``McbResult[T]`` (alias ``r``) — explicit fallible results.
- ``BaseMcbSettings`` — Pydantic ``BaseSettings`` with singleton lifecycle.
- ``McbService`` (alias ``s``) — per-class singleton service base.
- ``get_logger`` / ``configure_logging`` — structured logging.

Concrete implementations live in sibling modules so each module owns one
public abstraction (``result``, ``settings``, ``logger``, ``service``).
"""

from __future__ import annotations

from mcb_scripts.constants import McbConstants, c
from mcb_scripts.logger import configure_logging, get_logger
from mcb_scripts.result import McbResult, r
from mcb_scripts.service import McbScriptsService, McbService, s
from mcb_scripts.settings import BaseCommandSettings, BaseMcbSettings

__all__ = [
    "BaseCommandSettings",
    "BaseMcbSettings",
    "McbConstants",
    "McbResult",
    "McbScriptsService",
    "McbService",
    "c",
    "configure_logging",
    "get_logger",
    "r",
    "s",
]
