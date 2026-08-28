"""Shared pytest fixtures for scripts/lib tests.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from collections.abc import Callable, Generator, MutableMapping
from pathlib import Path
from typing import Any, cast

import pytest
from pydantic import create_model
from structlog.testing import capture_logs

from lib.core import BaseMcbSettings


@pytest.fixture(autouse=True)
def reset_settings() -> Generator[None]:
    """Reset the FLEXT settings singleton before and after every test."""
    BaseMcbSettings.reset_for_testing()
    yield
    BaseMcbSettings.reset_for_testing()


@pytest.fixture
def settings_factory() -> Callable[..., BaseMcbSettings]:
    """Return a factory that creates a fresh settings subclass instance."""

    def _make(**fields: Any) -> BaseMcbSettings:
        defs: dict[str, Any] = {
            name: (type(value), value) for name, value in fields.items()
        }
        _Settings = cast(
            type[BaseMcbSettings],
            create_model("_Settings", __base__=BaseMcbSettings, **defs),
        )
        _Settings.model_rebuild()
        return _Settings.fetch_global()

    return _make


@pytest.fixture
def temp_dir(tmp_path: Path) -> Path:
    """Provide a temporary directory for the current test."""
    return tmp_path


@pytest.fixture
def temp_file(temp_dir: Path) -> Callable[[str, str], Path]:
    """Return a helper that creates a temporary file with the given content."""

    def _make(filename: str, content: str) -> Path:
        path = temp_dir / filename
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    return _make


@pytest.fixture
def capture_logs_fixture() -> Generator[list[MutableMapping[str, Any]]]:
    """Capture structlog emissions during a test."""
    with capture_logs() as logs:
        yield logs
