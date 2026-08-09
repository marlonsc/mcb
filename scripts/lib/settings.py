"""FLEXT-style settings base for MCB Python tooling.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import ClassVar

from flext_core import FlextSettings
from pydantic import Field
from pydantic_settings import SettingsConfigDict

from .constants import c


class BaseMcbSettings(FlextSettings):
    """Base settings for MCB scripts with per-class singleton lifecycle.

    Environment variables are read with the ``MCB_`` prefix, e.g.
    ``MCB_LOG_LEVEL`` maps to ``log_level``.
    """

    model_config: ClassVar[SettingsConfigDict] = (
        FlextSettings.model_config.copy()
    )
    model_config["env_prefix"] = c.ENV_PREFIX
    model_config["extra"] = "ignore"
    model_config["validate_assignment"] = True

    @classmethod
    def resolve_env_file(cls, namespace: str | None = None) -> str:
        """Centralised ``.env`` discovery for MCB.

        Honours ``MCB_ENV_FILE``; otherwise prefers ``.env.mcb-{namespace}``
        when ``namespace`` is given and the file exists, falling back to ``.env``.
        """
        custom_env_file = os.environ.get(c.ENV_FILE_ENV_VAR)
        if custom_env_file:
            custom_path = Path(custom_env_file)
            if custom_path.exists():
                return str(custom_path.resolve())
            return custom_env_file
        if namespace:
            scoped = Path.cwd() / f".env.mcb-{namespace}"
            if scoped.exists():
                return str(scoped.resolve())
        default_path = Path.cwd() / ".env"
        if default_path.exists():
            return str(default_path.resolve())
        return c.ENV_FILE_DEFAULT


class BaseCommandSettings(BaseMcbSettings):
    """Base settings for cosmos-command scripts using unprefixed env vars.

    The cosmos-command dispatcher exposes ``WHAT``, ``ACT``, ``APPLY`` and
    other parameters without a prefix, so this base disables the default
    ``MCB_`` prefix while keeping the rest of the FLEXT settings lifecycle.
    """

    model_config: ClassVar[SettingsConfigDict] = (
        BaseMcbSettings.model_config.copy()
    )
    model_config["env_prefix"] = ""


class McbSettings(BaseMcbSettings):
    """Shared MCB settings with configurable project paths.

    These fields are intentionally overridable through ``MCB_*`` environment
    variables so that CI, local checkouts, and test workspaces can relocate
    inputs/outputs without editing source.
    """

    project_root: Path = Field(
        default=Path("."),
        description="Project root directory",
    )
    k8s_dir: Path = Field(
        default=Path("k8s"),
        description="Kubernetes manifests directory",
    )
    docs_dir: Path = Field(
        default=Path("docs"),
        description="Documentation directory",
    )
    qlty_check_sarif: Path = Field(
        default=Path("qlty.check.current.sarif"),
        description="SARIF output path for qlty check",
    )
    qlty_smells_sarif: Path = Field(
        default=Path("qlty.smells.sarif"),
        description="SARIF output path for qlty smells",
    )
    qlty_report_md: Path = Field(
        default=Path("QUALITY_REPORT.md"),
        description="Markdown quality report output path",
    )


__all__ = ["BaseMcbSettings", "BaseCommandSettings", "McbSettings"]
