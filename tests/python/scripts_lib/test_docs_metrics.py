"""Generated metrics follow source changes and reach a stable second run."""

from __future__ import annotations

import tomllib
from pathlib import Path

from mcb_scripts.docs.metrics import generate_metrics
from mcb_scripts.docs.utils import get_project_root


def test_generate_metrics_tracks_declared_sources_and_is_idempotent(
    tmp_path: Path,
) -> None:
    """A source change updates the document once; repeated runs retain its bytes."""
    project = get_project_root()
    config_source = project / "config/docs-metrics.toml"
    config = tomllib.loads(config_source.read_text(encoding="utf-8"))
    target_config = tmp_path / "config/docs-metrics.toml"
    target_config.parent.mkdir(parents=True)
    target_config.write_bytes(config_source.read_bytes())
    template = project / "docs/templates/metrics.md.j2"
    target_template = tmp_path / "docs/templates/metrics.md.j2"
    target_template.parent.mkdir(parents=True)
    target_template.write_bytes(template.read_bytes())
    (tmp_path / "Cargo.toml").write_text(
        '[workspace]\nmembers = []\n[workspace.package]\nversion = "1.2.3"\n',
        encoding="utf-8",
    )
    source_root = tmp_path / config["source_root"]
    source_root.mkdir()
    for category in config["categories"].values():
        directory = tmp_path / category["root"]
        directory.mkdir(parents=True)
        (directory / "sample.rs").write_text(
            "#[test]\nfn sample() {}\n", encoding="utf-8"
        )
    (tmp_path / config["adr_dir"]).mkdir()
    (tmp_path / config["module_docs_dir"]).mkdir()

    output, changed = generate_metrics(tmp_path)
    assert changed
    first = output.read_bytes()
    assert "Current Metrics (v1.2.3)" in first.decode("utf-8")
    assert generate_metrics(tmp_path) == (output, False)
    assert output.read_bytes() == first

    (source_root / "more.rs").write_text("fn more() {}\n", encoding="utf-8")
    assert generate_metrics(tmp_path) == (output, True)
    assert output.read_bytes() != first
