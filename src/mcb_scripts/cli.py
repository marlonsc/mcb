"""Repository automation CLI exposed by the generated package entry point.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_cli import cli

from mcb_scripts import McbScriptsModels, m, p, r
from mcb_scripts.docs.adr import create_adr
from mcb_scripts.docs.metrics import generate_metrics


def _create_adr(request: McbScriptsModels.McbScripts.AdrInput) -> p.Result[str]:
    destination = create_adr(
        request.root,
        request.title,
        request.status,
        dry_run=request.dry_run,
    )
    verb = "Would create" if request.dry_run else "Created"
    return r[str].ok(f"{verb} {destination}")


def _generate_metrics(
    request: McbScriptsModels.McbScripts.MetricsInput,
) -> p.Result[str]:
    destination, changed = generate_metrics(request.root)
    state = "updated" if changed else "unchanged"
    return r[str].ok(f"{destination}: {state}")


def main() -> int:
    """Register typed commands and execute the public CLI.

    Returns:
        The resulting ``int``.
    """
    app = cli.create_app_with_common_params(
        name="mcb-scripts",
        help_text="MCB repository automation",
    )
    cli.register_result_command(
        app,
        route=m.Cli.ResultCommandRoute(
            name="adr",
            help_text="Create an architecture decision record",
            model_cls=McbScriptsModels.McbScripts.AdrInput,
            handler=_create_adr,
        ),
    )
    cli.register_result_command(
        app,
        route=m.Cli.ResultCommandRoute(
            name="metrics",
            help_text="Generate deterministic project metrics",
            model_cls=McbScriptsModels.McbScripts.MetricsInput,
            handler=_generate_metrics,
        ),
    )
    return cli.finalize_result(cli.execute_app(app, prog_name="mcb-scripts"))


if __name__ == "__main__":
    raise SystemExit(main())
