"""MCB Python tooling constants.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import Final


class McbConstants:
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


c = McbConstants()

__all__ = ["McbConstants", "c"]
