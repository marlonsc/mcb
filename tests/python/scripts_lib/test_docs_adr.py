"""Public ADR creation behavior against the repository template."""

from __future__ import annotations

from pathlib import Path

import pytest

from mcb_scripts.docs.adr import create_adr
from mcb_scripts.docs.utils import get_project_root


def test_create_adr_renders_and_publishes_exclusively(tmp_path: Path) -> None:
    """Sequential records use the template and never overwrite existing output."""
    template = get_project_root() / "docs/templates/adr-template.md"
    target_template = tmp_path / "docs/templates/adr-template.md"
    target_template.parent.mkdir(parents=True)
    target_template.write_bytes(template.read_bytes())
    (tmp_path / "docs/adr").mkdir()

    first = create_adr(tmp_path, "Typed Template / Decision", "Accepted")
    content = first.read_text(encoding="utf-8")
    assert "# ADR 001: Typed Template / Decision" in content
    assert "## Status\n\nAccepted" in content
    assert first.name == "001-typed-template-decision.md"

    second = create_adr(tmp_path, "Another Decision", "Proposed")
    assert second.name == "002-another-decision.md"
    preview = create_adr(tmp_path, "Preview", "Proposed", dry_run=True)
    assert preview.name == "003-preview.md"
    assert not preview.exists()
    assert first.read_text(encoding="utf-8") == content


def test_create_adr_rejects_invalid_status_before_publication(tmp_path: Path) -> None:
    """Invalid user input cannot create an ADR artifact."""
    with pytest.raises(ValueError, match="Invalid ADR status"):
        create_adr(tmp_path, "Decision", "Unknown")
    assert not list(tmp_path.rglob("*.md"))
