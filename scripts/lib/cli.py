"""Minimal Typer CLI helpers aligned with FLEXT Result patterns.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import inspect
from collections.abc import Callable
from typing import Any

import typer
from flext_core import p
from pydantic import BaseModel


def create_app_with_common_params(
    *, name: str, help_text: str, add_completion: bool = False
) -> typer.Typer:
    """Create a Typer app with MCB CLI conventions."""
    return typer.Typer(name=name, help=help_text, add_completion=add_completion)


def register_result_command[P: BaseModel, T](
    app: typer.Typer,
    *,
    name: str,
    help_text: str,
    model_cls: type[P],
    handler: Callable[[P], p.Result[T]],
    success_message: str | None = None,
) -> None:
    """Register a model-backed command that returns a ``p.Result[T]``."""
    parameters: list[inspect.Parameter] = []
    annotations: dict[str, type[Any]] = {"return": type(None)}

    for field_name, field_info in model_cls.model_fields.items():
        if getattr(field_info, "exclude", None) is True:
            continue
        annotation = field_info.annotation or str
        if field_info.is_required():
            option = typer.Option(..., help=field_info.description)
        elif field_info.default_factory is not None:
            # Why: typer accepts default_factory at runtime (Option param),
            # but its distribution stubs expose no matching overload.
            option = typer.Option(  # pyrefly: ignore[no-matching-overload]
                default_factory=field_info.default_factory,
                help=field_info.description,
            )
        else:
            option = typer.Option(field_info.default, help=field_info.description)
        parameters.append(
            inspect.Parameter(
                field_name,
                kind=inspect.Parameter.KEYWORD_ONLY,
                default=option,
                annotation=annotation,
            )
        )
        annotations[field_name] = annotation

    def command(**kwargs: Any) -> None:
        params = model_cls.model_validate(kwargs)
        result = handler(params)
        if result.failure:
            typer.echo(f"Error: {result.error}", err=True)
            raise typer.Exit(code=1)
        if success_message is not None:
            typer.echo(success_message)

    command.__signature__ = inspect.Signature(parameters)  # pyright: ignore[reportFunctionMemberAccess]  # pyrefly: ignore[missing-attribute]
    command.__annotations__ = annotations
    app.command(name=name, help=help_text)(command)


def run_app(app: typer.Typer) -> None:
    """Invoke a Typer app via the shared dispatcher.

    Uses ``typer.main.get_command`` so that single-command apps retain their
    command-group structure (avoiding Typer's flattening behaviour that would
    reject the subcommand name as an unexpected positional argument).  This
    keeps entrypoint scripts free of a direct ``import typer``.
    """
    typer.main.get_command(app)()


__all__ = ["create_app_with_common_params", "register_result_command", "run_app"]
