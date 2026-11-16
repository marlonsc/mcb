"""Domain models for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
src/mcb_scripts/models
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

from flext_cli import FlextCliModels
from mcb_scripts import c


class McbScriptsModels(FlextCliModels):
    """Domain models for mcbscripts."""

    class McbScripts:
        """Declarations required by MCB's own documentation."""

        class MetricsCategory(FlextCliModels.Value):
            """One configured Rust source category."""

            root: str = FlextCliModels.Field(description="Declared source directory.")
            exclude: tuple[str, ...] = FlextCliModels.Field(
                default_factory=tuple,
                description="Excluded source file stems.",
            )
            labels: Mapping[str, str] = FlextCliModels.Field(
                default_factory=lambda: c.Cli.EMPTY_STR_MAPPING,
                description="Display labels keyed by source stem.",
            )

        class MetricsConfig(FlextCliModels.Value):
            """Validated input for the MCB metrics document."""

            categories: Mapping[
                str,
                McbScriptsModels.McbScripts.MetricsCategory,
            ] = FlextCliModels.Field(description="Declared code categories.")
            source_root: str = FlextCliModels.Field(description="Rust source root.")
            adr_dir: str = FlextCliModels.Field(description="ADR directory.")
            module_docs_dir: str = FlextCliModels.Field(
                description="Module documentation directory."
            )

        class AdrInput(FlextCliModels.Value):
            """Inputs for MCB's architecture decision document."""

            title: str = FlextCliModels.Field(description="Single-line ADR title.")
            status: str = FlextCliModels.Field(
                default="Proposed", description="ADR status."
            )
            dry_run: bool = FlextCliModels.Field(
                default=False, description="Preview publication."
            )
            root: Path = FlextCliModels.Field(
                default=Path(), description="Repository root."
            )

        class MetricsInput(FlextCliModels.Value):
            """Repository selected for the MCB metrics document."""

            root: Path = FlextCliModels.Field(
                default=Path(), description="Repository root."
            )

        class MetricsContext(FlextCliModels.Value):
            """Measured MCB document values supplied to the shared renderer."""

            version: str = FlextCliModels.Field(description="Rust workspace version.")
            version_anchor: str = FlextCliModels.Field(description="Version link anchor.")
            table: str = FlextCliModels.Field(description="Measured Markdown table.")
            language_list: str = FlextCliModels.Field(description="Supported languages.")
            embedding_list: str = FlextCliModels.Field(description="Embedding providers.")
            vector_store_list: str = FlextCliModels.Field(description="Vector stores.")

m = McbScriptsModels

__all__: list[str] = ["McbScriptsModels", "m"]
