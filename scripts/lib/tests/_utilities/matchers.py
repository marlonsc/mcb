"""Test matchers for MCB Python tooling tests.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import Any

from flext_core import p


class TestMatchers:
    """Fluent assertions for common MCB test patterns."""

    @staticmethod
    def ok(result: p.Result[Any], value: Any | None = None) -> None:
        """Assert ``result`` is successful and optionally carries ``value``."""
        assert result.success, f"expected success, got failure: {result.error}"
        if value is not None:
            assert result.unwrap() == value, (
                f"expected {value!r}, got {result.unwrap()!r}"
            )

    @staticmethod
    def fail(result: p.Result[Any], contains: str | None = None) -> None:
        """Assert ``result`` is a failure and optionally contains ``contains``."""
        assert result.failure, f"expected failure, got success: {result.unwrap()!r}"
        if contains is not None:
            error = result.error or ""
            assert contains in error, (
                f"expected error to contain {contains!r}, got {error!r}"
            )

    @staticmethod
    def that(condition: bool, message: str = "assertion failed") -> None:
        """Generic assertion wrapper with a human-readable message."""
        assert condition, message


tm = TestMatchers()

__all__ = ["TestMatchers", "tm"]
