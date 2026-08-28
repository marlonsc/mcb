"""FLEXT-style service base for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import Annotated

from flext_core import FlextService
from pydantic import Field

from .settings import BaseMcbSettings


class McbScriptsService(FlextService):
    """Per-class singleton service base wired to MCB settings."""

    settings_type: Annotated[
        type | None,
        Field(
            exclude=True, description="Settings class used to initialize the service."
        ),
    ] = BaseMcbSettings


McbService = McbScriptsService
s = McbScriptsService

__all__ = ["McbScriptsService", "McbService", "s"]
