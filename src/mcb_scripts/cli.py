"""Repository automation CLI exposed by the generated package entry point."""

from __future__ import annotations

from flext_cli import cli

from flext_core import m, p, r

from .docs.adr import create_adr
from .docs.metrics import generate_metrics
from .docs.utils import get_project_root


class AdrInput(m.Value):
    """Validated request for a new architecture decision record."""

    title: str = m.Field(description="Single-line ADR title.")
    status: str = m.Field(default="Proposed", description="ADR lifecycle status.")
    dry_run: bool = m.Field(default=False, description="Preview without publishing.")


class MetricsInput(m.Value):
    """No caller values are needed to generate repository metrics."""


def _create_adr(request: AdrInput) -> p.Result[str]:
    destination = create_adr(
        get_project_root(), request.title, request.status, dry_run=request.dry_run
    )
    verb = "Would create" if request.dry_run else "Created"
    return r[str].ok(f"{verb} {destination}")


def _generate_metrics(_request: MetricsInput) -> p.Result[str]:
    destination, changed = generate_metrics(get_project_root())
    state = "updated" if changed else "unchanged"
    return r[str].ok(f"{destination}: {state}")


def main() -> int:
    """Register typed commands and execute the public CLI."""
    app = cli.create_app_with_common_params(
        name="mcb-scripts", help_text="MCB repository automation"
    )
    cli.register_result_command(
        app,
        name="adr",
        help_text="Create an architecture decision record",
        model_cls=AdrInput,
        handler=_create_adr,
    )
    cli.register_result_command(
        app,
        name="metrics",
        help_text="Generate deterministic project metrics",
        model_cls=MetricsInput,
        handler=_generate_metrics,
    )
    return cli.finalize_result(cli.execute_app(app, prog_name="mcb-scripts"))


if __name__ == "__main__":
    raise SystemExit(main())
