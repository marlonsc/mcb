"""Generate the deterministic project metrics page from declared source roots.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import re
from pathlib import Path

from mcb_scripts import McbScriptsModels, u

_TEST = re.compile(r"#\[\s*(?:tokio::)?test\s*\]")
_ADR = re.compile(r"^[0-9]{3}-[a-z0-9-]+\.md$")


def _declared_dir(root: Path, value: str) -> Path:
    """Resolve only a declared directory inside this repository.

    Returns:
        The resulting ``Path``.

    Raises:
        FileNotFoundError: If ``not directory.is_dir()``.
        ValueError: If Docs metrics source path escapes the repository.
    """
    relative = Path(value)
    if relative.is_absolute() or ".." in relative.parts:
        msg = f"Docs metrics source path escapes the repository: {value!r}"
        raise ValueError(msg)
    directory = root / relative
    if not directory.is_dir():
        raise FileNotFoundError(directory)
    return directory


def _category(
    root: Path,
    raw: McbScriptsModels.McbScripts.MetricsCategory,
) -> tuple[str, ...]:
    """Resolve one configured code category without inventing missing sources.

    Returns:
        The resulting ``tuple[str, ...]``.
    """
    directory = _declared_dir(root, raw.root)
    names = (
        raw.labels.get(path.stem, path.stem.replace("_", " ").title())
        for path in sorted(directory.glob("*.rs"))
        if path.stem not in raw.exclude
    )
    return tuple(names)


def _metrics_inputs(
    root: Path,
) -> tuple[
    McbScriptsModels.McbScripts.MetricsConfig,
    Path,
    Path,
    Path,
]:
    """Load the declared metrics config and resolve its directories.

    Returns:
        The resulting ``tuple[McbScriptsMetricsConfig, Path, Path, Path]``.
    """
    config_path = root / "config/docs-metrics.toml"
    config = McbScriptsModels.McbScripts.MetricsConfig.model_validate_json(
        u.Cli.json_dumps(
            dict(u.Cli.toml_read_json(config_path).unwrap()),
        ).unwrap(),
    )
    return (
        config,
        _declared_dir(root, config.source_root),
        _declared_dir(root, config.adr_dir),
        _declared_dir(root, config.module_docs_dir),
    )


def _rust_stats(
    source_root: Path,
) -> tuple[tuple[Path, ...], tuple[Path, ...], int, int]:
    """Collect Rust source files, test files, test count, and source lines.

    Returns:
        The resulting ``tuple[tuple[Path, ...], tuple[Path, ...], int, int]``.

    Raises:
        ValueError: If No Rust source files under; or if No Rust tests found in
            the declared source root.
    """
    source_files = tuple(sorted(source_root.rglob("*.rs")))
    if not source_files:
        msg = f"No Rust source files under {source_root}"
        raise ValueError(msg)
    test_files = tuple(
        path for path in source_files if "tests" in path.relative_to(source_root).parts
    )
    test_count = 0
    source_lines = 0
    for path in source_files:
        source = path.read_text(encoding="utf-8")
        test_count += len(_TEST.findall(source))
        source_lines += len(source.splitlines())
    if test_count == 0:
        msg = "No Rust tests found in the declared source root"
        raise ValueError(msg)
    return source_files, test_files, test_count, source_lines


def _workspace_version(root: Path) -> str:
    """Read the workspace package version from Cargo.toml.

    Returns:
        The resulting ``str``.

    Raises:
        TypeError: If Workspace version must be a nonempty string.
    """
    cargo = u.Cli.toml_read_json(root / "Cargo.toml").unwrap()
    version = u.Cli.json_walk_path(cargo, ("workspace", "package", "version"))
    if not isinstance(version, str) or not version:
        msg = "Workspace version must be a nonempty string"
        raise TypeError(msg)
    return version


def _metric_table_lines(values: tuple[tuple[str, str], ...]) -> list[str]:
    """Render the aligned markdown metric table body.

    Returns:
        The resulting ``list[str]``.
    """
    label_width = max(len(label) for label, _ in values)
    value_width = max(5, *(len(value) for _, value in values))
    return [
        f"| {'Metric':<{label_width}} | {'Value':>{value_width}} |",
        f"| {'-' * label_width} | {'-' * (value_width - 1)}: |",
        *(
            f"| {label:<{label_width}} | {value:>{value_width}} |"
            for label, value in values
        ),
    ]


def render_metrics(root: Path) -> str:
    """Render the complete metrics document from live Rust and docs sources.

    Returns:
        The resulting ``str``.
    """
    config, source_root, adr_dir, module_docs_dir = _metrics_inputs(root)
    source_files, test_files, test_count, source_lines = _rust_stats(source_root)
    version = _workspace_version(root)

    languages = _category(root, config.categories["language"])
    embeddings = _category(root, config.categories["embedding"])
    vector_stores = _category(root, config.categories["vector_store"])
    values = (
        ("Version", version),
        ("Languages", str(len(languages))),
        ("Embedding Providers", str(len(embeddings))),
        ("Vector Stores", str(len(vector_stores))),
        (
            "ADRs",
            str(sum(1 for path in adr_dir.iterdir() if _ADR.fullmatch(path.name))),
        ),
        ("Tests", str(test_count)),
        ("Source Files", str(len(source_files))),
        ("Source Lines", str(source_lines)),
        ("Test Files", str(len(test_files))),
        ("Module Docs", str(len(tuple(module_docs_dir.glob("*.md"))))),
    )
    context = McbScriptsModels.McbScripts.MetricsContext(
        version=version,
        version_anchor="v" + re.sub(r"[^a-z0-9]", "", version.lower()),
        table="\n".join(_metric_table_lines(values)),
        language_list="\n".join(f"- {name}" for name in languages),
        embedding_list="\n".join(f"- {name}" for name in embeddings),
        vector_store_list="\n".join(f"- {name}" for name in vector_stores),
    )
    content = u.Cli.template_render(
        root / "docs/templates/metrics.md.j2",
        context,
    ).unwrap()
    return content.rstrip() + "\n"


def generate_metrics(root: Path) -> tuple[Path, bool]:
    """Publish changed bytes atomically and report whether output changed.

    Returns:
        The resulting ``tuple[Path, bool]``.
    """
    destination = root / "docs/generated/METRICS.md"
    content = render_metrics(root)
    before = destination.read_text(encoding="utf-8") if destination.exists() else None
    if before == content:
        return destination, False
    u.Cli.atomic_write_text_file(destination, content).unwrap()
    return destination, True
