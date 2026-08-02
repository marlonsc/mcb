"""Tests for scripts.lib.cli.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import pytest
import typer
from pydantic import BaseModel
from typer.testing import CliRunner

from ..cli import create_app_with_common_params, register_result_command
from ..core import r
from ._utilities.matchers import tm


class _DummyParams(BaseModel):
    name: str = "world"


def _hello_handler(params: _DummyParams) -> r[str]:
    return r[str].ok(f"Hello, {params.name}!")


def _bye_handler(params: _DummyParams) -> r[str]:
    return r[str].ok(f"Bye, {params.name}!")


def _fail_handler(params: _DummyParams) -> r[str]:
    return r[str].fail("boom")


def _build_app() -> typer.Typer:
    app = create_app_with_common_params(name="test", help_text="A test app")
    register_result_command(
        app,
        name="hello",
        help_text="Say hello",
        model_cls=_DummyParams,
        handler=_hello_handler,
        success_message="Greeting delivered",
    )
    register_result_command(
        app,
        name="bye",
        help_text="Say bye",
        model_cls=_DummyParams,
        handler=_bye_handler,
    )
    register_result_command(
        app,
        name="fail",
        help_text="Always fail",
        model_cls=_DummyParams,
        handler=_fail_handler,
    )
    return app


def test_create_app_returns_typer_app() -> None:
    """create_app_with_common_params returns a Typer application."""
    app = create_app_with_common_params(name="test", help_text="A test app")
    tm.that(isinstance(app, typer.Typer), "expected a typer.Typer instance")


def test_register_result_command() -> None:
    """register_result_command wires a model-backed handler into the app."""
    app = _build_app()
    runner = CliRunner()
    invocation = runner.invoke(app, ["hello", "--name", "Flext"])
    tm.that(invocation.exit_code == 0, f"unexpected exit: {invocation.output}")
    tm.that("Greeting delivered" in invocation.output, "expected success message")


def test_result_command_failure_exits_non_zero() -> None:
    """A failing result command exits with code 1."""
    app = _build_app()
    runner = CliRunner()
    invocation = runner.invoke(app, ["fail"])
    tm.that(invocation.exit_code == 1, f"expected failure exit: {invocation.output}")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
